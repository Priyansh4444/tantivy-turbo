// Frozen workload used for both revisions. Run outside the timed sections to
// construct one index, then reopen that same index for every measurement.
use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};
use tantivy::collector::{Count, TopDocs};
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, TEXT};
use tantivy::{doc, Index, ReloadPolicy};

const CASES: [(&str, &str, bool); 6] = [
    ("term_top10", "alpha", true),
    ("and_top10", "+alpha +beta", true),
    ("or_top10", "alpha OR beta OR gamma", true),
    ("phrase_top10", "\"alpha beta\"", true),
    ("and_count", "+alpha +beta", false),
    ("or_count", "alpha OR beta OR gamma", false),
];

fn main() -> tantivy::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 || !["build", "measure"].contains(&args[1].as_str()) {
        eprintln!("usage: harness build|measure INDEX_DIR");
        std::process::exit(2);
    }
    let path = Path::new(&args[2]);
    if args[1] == "build" {
        std::fs::create_dir_all(path)?;
        let mut builder = Schema::builder();
        let body = builder.add_text_field("body", TEXT);
        let index = Index::create_in_dir(path, builder.build())?;
        let mut writer = index.writer_with_num_threads(1, 50_000_000)?;
        // Deterministic mix of selectivities and term frequencies.
        for i in 0..80_000_u32 {
            let mut text = String::from("filler document");
            if i % 2 == 0 { text.push_str(" alpha"); }
            if i % 5 == 0 { text.push_str(" beta"); }
            if i % 11 == 0 { text.push_str(" gamma"); }
            if i % 23 == 0 { text.push_str(" alpha beta"); }
            if i % 37 == 0 { text.push_str(" alpha alpha alpha"); }
            writer.add_document(doc!(body => text))?;
        }
        writer.commit()?;
        return Ok(());
    }
    let index = Index::open_in_dir(path)?;
    let field = index.schema().get_field("body")?;
    let reader = index.reader_builder().reload_policy(ReloadPolicy::Manual).try_into()?;
    let searcher = reader.searcher();
    let parser = QueryParser::for_index(&index, vec![field]);
    for (name, query_text, top) in CASES {
        let query = parser.parse_query(query_text)?;
        // Fingerprint full score bits and addresses for top-k, or exact counts.
        // This catches behavioral changes on this workload independently of speed.
        let checksum = if top {
            let hits = searcher.search(&query, &TopDocs::with_limit(10).order_by_score())?;
            let mut h = 14695981039346656037_u64;
            for (score, address) in hits {
                for v in [score.to_bits(), address.segment_ord, address.doc_id] {
                    h = (h ^ u64::from(v)).wrapping_mul(1099511628211);
                }
            }
            h
        } else {
            searcher.search(&query, &Count)? as u64
        };
        // Warm up, then measure at least 150ms per case to reduce timer noise.
        for _ in 0..10 {
            if top { black_box(searcher.search(&query, &TopDocs::with_limit(10).order_by_score())?); }
            else { black_box(searcher.search(&query, &Count)?); }
        }
        let start = Instant::now();
        let mut n = 0_u64;
        while start.elapsed() < Duration::from_millis(150) {
            if top { black_box(searcher.search(&query, &TopDocs::with_limit(10).order_by_score())?); }
            else { black_box(searcher.search(&query, &Count)?); }
            n += 1;
        }
        println!("{} {} {} {}", name, start.elapsed().as_nanos(), n, checksum);
    }
    Ok(())
}
