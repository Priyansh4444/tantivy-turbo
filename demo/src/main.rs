use fframes::{EncoderOptions, RenderOptions, StaticMediaProvider, cli};
use fframes_skia_renderer::{
    SkiaFFramesRenderer, SkiaPipelineConcurrencyPolicy, SkiaPipelineConfig,
    vulkan::SkiaVulkanCtx,
};
use tantivy_turbo_demo::{ TantivyTurboDemoMedia, TantivyTurboDemoVideo, HEIGHT, WIDTH };
use fframes::cli::clap; // the derive below expands to `clap::...`
use std::process::ExitCode;

/// This video has no flags of its own; `fframes::cli` adds render, frame, strip, inspect, ...
/// Run `cargo run --release -- --help`.
#[derive(Debug, clap::Args)]
struct VideoArgs {}

fn main() -> ExitCode {
    let args = cli::parse::<VideoArgs>();
    let media = TantivyTurboDemoMedia::prepare().expect("media");
    let video = TantivyTurboDemoVideo::new(&media);
    let gpu = SkiaVulkanCtx::new(WIDTH, HEIGHT).expect("GPU context");

    cli::new(
        &video,
        RenderOptions {
            media: Some(&media),
            video_encoder_options: EncoderOptions {
                // VP9 (WebM) links against the system libvpx; transcode to MP4/GIF afterwards
                // with any ffmpeg if needed (see README).
                preferred_encoder: Some("libvpx-vp9"),
                codec_params: Some(&[("crf", "24"), ("b", "0"), ("row-mt", "1"), ("deadline", "good"), ("cpu-used", "4")]),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .args(args)
    // Frame previews (frame, strip, onion, snapshot) render with the same Skia backend.
    .backend(
        SkiaFFramesRenderer::new_vulkan(
            &gpu,
            SkiaPipelineConfig {
                concurrency_policy: SkiaPipelineConcurrencyPolicy::MaxPerformance,
                ..Default::default()
            },
        )
        .expect("skia renderer"),
    )
    // `preview` opens the real-time GPU player.
    .preview(fframes_native_player::cli_preview)
    .run()
}
