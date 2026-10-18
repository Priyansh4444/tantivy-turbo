# tantivy-turbo demo video

A 32-second 1080p launch video for tantivy-turbo, written in Rust with
[fframes](https://github.com/dmtrKovalenko/fframes): six scenes, Bayer-dithered
bars and backgrounds (16 fill levels per color as SVG patterns), no audio. Every
number on screen comes from the benchmarks in the repository README.

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
