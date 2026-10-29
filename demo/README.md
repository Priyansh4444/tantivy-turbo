# tantivy-turbo demo video

A 21-second, 60-fps 1080p launch video for tantivy-turbo, with no audio.
It uses one continuous timeline; the search example is illustrative and the
comparison is from the expanded October 28 benchmark.

```bash
cargo build --release                         # first build downloads prebuilt Skia/ffmpeg
R() { cargo run --release -- "$@"; }
R timeline                                    # scenes and timings
R inspect --fail-on warning                   # checks every 0.25 s for layout problems
R strip all -n 8                              # contact sheet of the video
R preview                                     # real-time GPU window
R render -o demo.mkv                          # VP9 in Matroska
ffmpeg -i demo.mkv -c:v libx264 -crf 20 -pix_fmt yuv420p demo.mp4
```

The project uses the VP9 encoder (`vpx` feature) because the prebuilt H.264 path
needs a matching libx264 build; transcode with any ffmpeg afterwards.
`tantivy-turbo-demo.mp4`, `poster.png` and `bars.gif` in this folder are the
rendered outputs.

## Transcript

- An illustrative search finds and ranks documents about quokkas. The document
  titles are fictional; the visual sequence
  simplifies how the index performs that work.
- For the best ten results on 1,201 warm port-style searches, the October 28
  comparison measured an average of the two order-specific mean query medians
  of about 136.2 µs for tantivy-turbo and 168.5 µs for lucene-rs: about 1.24×
  faster. The moving bars are a visualization, not a live recording. The run
  had background load and the APIs differ in their total-hit thresholds; see
  `../bench-results/expanded-port-20261028.md` for the method and limits.
- An earlier build was 1.82–2.09× faster by the geometric mean of query
  medians in the two process orders, on 20 warm top-10 searches versus Apache
  Lucene 10.4 (Java). This is the October 4 Wikipedia 1M run, not a guarantee
  for other workloads; see `../doc/performance/lucene-10.4/results/2026-10-04-native/README.md`.
  [Lucene is the search core](https://lucene.apache.org/) of Elasticsearch,
  OpenSearch and Solr. [GitHub's code search](https://github.blog/engineering/the-technology-behind-githubs-new-code-search/)
  uses its own Rust engine, Blackbird.
- A separate full-scan differential test passed 14,400 randomized query and
  scoring-configuration checks. These are checks, not necessarily distinct
  queries, and do not prove correctness for all workloads.
- The closing message highlights the fork's Tantivy API compatibility; the
  benchmark is evidence for the displayed workload, not a guarantee that every
  query will be faster.
