# tantivy-turbo demo video

A 45-second 1080p launch video for tantivy-turbo, written in Rust with
[fframes](https://github.com/dmtrKovalenko/fframes): eight scenes, solid bars,
and no audio. Benchmark figures are from the repository; the AND window-count
animation is illustrative.

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
