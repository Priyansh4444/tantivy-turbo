# tantivy-turbo demo video

A 32.5-second 1080p launch video for tantivy-turbo: six scenes, solid bars,
and no audio. Benchmark figures are from the repository; the search example
is illustrative.

```bash
cargo build --release                         # first build downloads prebuilt Skia/ffmpeg
R() { cargo run --release -- "$@"; }
R timeline                                    # scenes and timings
R inspect --fail-on warning                   # checks every 0.25 s for layout problems
R strip VsUpstream -n 8                       # contact sheet of one scene
R preview                                     # real-time GPU window
R render -o demo.mkv                          # VP9 in Matroska
ffmpeg -i demo.mkv -c:v libx264 -crf 20 -pix_fmt yuv420p demo.mp4
```

The project uses the VP9 encoder (`vpx` feature) because the prebuilt H.264 path
needs a matching libx264 build; transcode with any ffmpeg afterwards.
`tantivy-turbo-demo.mp4`, `poster.png` and `bars.gif` in this folder are the
rendered outputs.

## Transcript

- Search finds matches in a collection. The demo compares finding the best ten
  results and counting all matches.
- On 1,201 benchmark searches, the recorded run was 1.79× faster for the best
  ten results and 1.59× faster for counts versus Tantivy 0.27. Against lucene-rs,
  it was 1.25× faster for the best ten overall and 1.04× on searches with three
  or more alternatives. These are the October 23 results from before a later
  skip-reader fix, with both run orders combined.
- In an illustrative search for “quokka AND animal,” start with the rarer
  word and retain documents containing both words.
- Results were checked against a previous build, a full scan, and the
  lucene-rs port on the specified test workloads. The full benchmark details
  and limitations are in `../BENCHMARKS.md`.
