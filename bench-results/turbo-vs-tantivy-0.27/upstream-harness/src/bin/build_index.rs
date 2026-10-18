//! Index the aligned Wiki1M JSONL with upstream Tantivy into one segment.
use std::io::BufRead;
use tantivy::schema::{Schema, STORED, STRING, TEXT, FAST};
use tantivy::{Index, IndexWriter, TantivyDocument};

fn main() -> tantivy::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (corpus, out) = (&args[0], &args[1]);
    let mut builder = Schema::builder();
    let id = builder.add_text_field("id", STRING | STORED);
    let text = builder.add_text_field("text", TEXT);
    let sort = builder.add_u64_field("sort_field", FAST);
    let schema = builder.build();
    std::fs::create_dir_all(out)?;
    let index = Index::create_in_dir(out, schema)?;
    let mut writer: IndexWriter = index.writer_with_num_threads(4, 2_000_000_000)?;
    let file = std::io::BufReader::new(std::fs::File::open(corpus)?);
    for (n, line) in file.lines().enumerate() {
        let v: serde_json::Value = serde_json::from_str(&line?).unwrap();
        let mut doc = TantivyDocument::default();
        doc.add_text(id, v["id"].as_str().unwrap());
        doc.add_text(text, v["text"].as_str().unwrap());
        doc.add_u64(sort, v["sort_field"].as_u64().unwrap());
        writer.add_document(doc)?;
        if n % 100_000 == 0 { eprintln!("{n}"); }
    }
    writer.commit()?;
    let segments = index.searchable_segment_ids()?;
    if segments.len() > 1 {
        writer.merge(&segments).wait()?;
    }
    writer.wait_merging_threads()?;
    let index = Index::open_in_dir(out)?;
    eprintln!("segments {}", index.searchable_segment_ids()?.len());
    Ok(())
}
