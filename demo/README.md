# tantivy-turbo demo video

A 21.7-second 1080p launch video for tantivy-turbo: five scenes and no audio.
The search example is illustrative; the comparison is from the expanded
October 28 benchmark.

```bash
cargo build --release                         # first build downloads prebuilt Skia/ffmpeg
R() { cargo run --release -- "$@"; }
R timeline                                    # scenes and timings
R inspect --fail-on warning                   # checks every 0.25 s for layout problems
R strip Comparison -n 8                       # contact sheet of one scene
R preview                                     # real-time GPU window
R render -o demo.mkv                          # VP9 in Matroska
ffmpeg -i demo.mkv -c:v libx264 -crf 20 -pix_fmt yuv420p demo.mp4
```

The project uses the VP9 encoder (`vpx` feature) because the prebuilt H.264 path
needs a matching libx264 build; transcode with any ffmpeg afterwards.
`tantivy-turbo-demo.mp4`, `poster.png` and `bars.gif` in this folder are the
rendered outputs.

## Transcript

- An illustrative search finds documents containing both “quokka” and
  “animal.” The visual sequence simplifies how the index performs that work.
- For the best ten results on 1,201 warm port-style searches, the October 28
  comparison measured an average of the two order-specific mean query medians
  of about 136.2 µs for tantivy-turbo and 168.5 µs for lucene-rs: about 1.24×
  faster. The moving bars are a visualization, not a live recording. The run
  had background load and the APIs differ in their total-hit thresholds; see
  `../bench-results/expanded-port-20261028.md` for the method and limits.
- A separate full-scan differential test passed 14,400 randomized query and
  scoring-configuration checks. These are checks, not necessarily distinct
  queries, and do not prove correctness for all workloads.
