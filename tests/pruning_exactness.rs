//! Every pruned top-k path must return exactly what scoring every match
//! returns: the same documents and the same score bits, and counts must equal
//! the number of matches. Random multi-segment indexes with dense and sparse
//! terms, deletions, repeated query terms and non-default BM25 parameters.
use tantivy::collector::{Collector, Count, SegmentCollector, TopDocs};
use tantivy::query::{Bm25Parameters, BooleanQuery, Occur, PhraseQuery, Query, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, Schema, TEXT};
use tantivy::{
    doc, DocAddress, DocId, Index, Score, Searcher, SegmentOrdinal, SegmentReader, Term,
};

/// Scores every match through the unpruned `for_each` path.
struct AllScored;

struct AllScoredSegment {
    segment: SegmentOrdinal,
    hits: Vec<(Score, DocAddress)>,
}

impl Collector for AllScored {
    type Fruit = Vec<(Score, DocAddress)>;
    type Child = AllScoredSegment;

    fn for_segment(
        &self,
        segment: SegmentOrdinal,
        _reader: &SegmentReader,
    ) -> tantivy::Result<AllScoredSegment> {
        Ok(AllScoredSegment {
            segment,
            hits: Vec::new(),
        })
    }

    fn requires_scoring(&self) -> bool {
        true
    }

    fn merge_fruits(&self, fruits: Vec<Vec<(Score, DocAddress)>>) -> tantivy::Result<Self::Fruit> {
        Ok(fruits.into_iter().flatten().collect())
    }
}

impl SegmentCollector for AllScoredSegment {
    type Fruit = Vec<(Score, DocAddress)>;

    fn collect(&mut self, doc: DocId, score: Score) {
        self.hits.push((score, DocAddress::new(self.segment, doc)));
    }

    fn harvest(self) -> Self::Fruit {
        self.hits
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

const VOCABULARY: usize = 60;

fn word(rank: usize) -> String {
    format!("w{rank}")
}

/// A Zipf-like vocabulary: low ranks are dense, high ranks sparse.
fn build_index(seed: u64, delete: bool) -> tantivy::Result<(Index, Field)> {
    let mut schema = Schema::builder();
    let text = schema.add_text_field("text", TEXT);
    let index = Index::create_in_ram(schema.build());
    let mut writer = index.writer_with_num_threads(1, 50_000_000)?;
    let mut rng = Rng(seed);
    for segment in 0..3 {
        let docs = 6_000 + segment * 3_000;
        for _ in 0..docs {
            let length = 1 + rng.below(40) as usize;
            let mut words = Vec::with_capacity(length);
            for _ in 0..length {
                // Squaring a uniform draw skews toward low (dense) ranks.
                let draw = rng.below(1_000) as f64 / 1_000.0;
                words.push(word((draw * draw * VOCABULARY as f64) as usize));
            }
            writer.add_document(doc!(text => words.join(" ")))?;
        }
        writer.commit()?;
    }
    if delete {
        for rank in [3usize, 17, 41] {
            writer.delete_term(Term::from_field_text(text, &word(rank)));
        }
        writer.commit()?;
    }
    Ok((index, text))
}

fn term(text: Field, rank: usize) -> Box<dyn Query> {
    Box::new(TermQuery::new(
        Term::from_field_text(text, &word(rank)),
        IndexRecordOption::WithFreqsAndPositions,
    ))
}

fn random_query(rng: &mut Rng, text: Field) -> Box<dyn Query> {
    let rank = |rng: &mut Rng| rng.below(VOCABULARY as u64) as usize;
    match rng.below(5) {
        0 => term(text, rank(rng)),
        1 => {
            // Two to eight terms, possibly repeated.
            let clauses = (0..2 + rng.below(7))
                .map(|_| (Occur::Should, term(text, rank(rng))))
                .collect();
            Box::new(BooleanQuery::new(clauses))
        }
        2 => {
            let clauses = (0..2 + rng.below(3))
                .map(|_| (Occur::Must, term(text, rank(rng))))
                .collect();
            Box::new(BooleanQuery::new(clauses))
        }
        3 => {
            let terms = (0..2 + rng.below(2))
                .map(|_| Term::from_field_text(text, &word(rank(rng) % 12)))
                .collect();
            Box::new(PhraseQuery::new(terms))
        }
        _ => {
            let clauses = vec![
                (Occur::Must, term(text, rank(rng))),
                (Occur::Should, term(text, rank(rng))),
                (Occur::Should, term(text, rank(rng))),
            ];
            Box::new(BooleanQuery::new(clauses))
        }
    }
}

fn check(searcher: &Searcher, query: &dyn Query, label: &str) -> tantivy::Result<()> {
    let mut all = searcher.search(query, &AllScored)?;
    let count = searcher.search(query, &Count)?;
    assert_eq!(count, all.len(), "{label}: Count collector");
    assert_eq!(query.count(searcher)?, all.len(), "{label}: Query::count");
    // Highest score first; among equal scores the lower address wins.
    all.sort_by(|left, right| right.0.total_cmp(&left.0).then(left.1.cmp(&right.1)));
    for limit in [1, 10, 100] {
        let pruned = searcher.search(query, &TopDocs::with_limit(limit).order_by_score())?;
        let expected = &all[..limit.min(all.len())];
        let to_bits = |hits: &[(Score, DocAddress)]| -> Vec<(u32, DocAddress)> {
            hits.iter()
                .map(|&(score, doc)| (score.to_bits(), doc))
                .collect()
        };
        assert_eq!(to_bits(&pruned), to_bits(expected), "{label}: top {limit}");
    }
    Ok(())
}

#[test]
fn pruned_top_k_and_counts_match_exhaustive_scoring() -> tantivy::Result<()> {
    for (seed, delete) in [
        (0x1234_5678_9abc_def1u64, false),
        (0x0fed_cba9_8765_4321, true),
        (0x9e37_79b9_7f4a_7c15, false),
        (0x2545_f491_4f6c_dd1d, true),
    ] {
        let (index, text) = build_index(seed, delete)?;
        let base = index.reader()?.searcher();
        let searchers = [
            base.clone(),
            base.clone()
                .with_bm25_parameters(Bm25Parameters::new(0.9, 0.4).unwrap()),
            base.with_bm25_parameters(Bm25Parameters::new(2.0, 1.0).unwrap()),
        ];
        let mut rng = Rng(seed ^ 0x5555);
        // Fewer rounds when assertions and overflow checks slow every query.
        let rounds = if cfg!(debug_assertions) { 100 } else { 400 };
        for round in 0..rounds {
            let query = random_query(&mut rng, text);
            for (index, searcher) in searchers.iter().enumerate() {
                let label = format!("delete={delete} round={round} params={index} {query:?}");
                check(searcher, query.as_ref(), &label)?;
            }
        }
    }
    Ok(())
}
