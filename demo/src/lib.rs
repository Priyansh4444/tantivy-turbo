//! tantivy-turbo launch video: six short scenes rendered in Rust.
//!
//! Benchmark figures come from the repository (`bench-results/` and
//! `doc/performance/`); the search example is illustrative.
//! The look matches the README charts:
//! bright blue and cyan marks on black, with explanations for new viewers.
use fframes::{
    AnimateRuntimeInput, AudioMap, Color, Duration, FFramesContext, Frame, Overlap, Scene, Scenes,
    Svgr, Transform, Video,
    animation::{AnimationRuntime, Easing},
    include_media_dir,
};
use std::sync::LazyLock;

include_media_dir!(pub struct TantivyTurboDemoMedia, "media");

pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;

const BG: &str = "#000000";
const PANEL: &str = "#111315";
const LINE: &str = "#303439";
const INK: &str = "#f7f9f9";
const INK2: &str = "#ccd3da";
const MUTED: &str = "#9daab6";
const BLUE: &str = "#329cff";
const AQUA: &str = "#00d8c5";
const SANS: &str = "DM Sans";
const MONO: &str = "JetBrains Mono";
const W_SANS: u16 = 500;
const W_MONO: u16 = 400;

/// Seconds shared by adjacent scenes. Titles fade through the background so
/// two explanations are never superimposed during the transition.
const XFADE: f32 = 0.2;
const SCENES: [f32; 6] = [4.0, 6.0, 6.0, 8.0, 4.5, 4.0];

static EASE: LazyLock<AnimationRuntime> =
    LazyLock::new(|| AnimationRuntime::new(0.8, &Easing::CubicBezier(0.16, 1.0, 0.3, 1.0)));
static GROW: LazyLock<AnimationRuntime> =
    LazyLock::new(|| AnimationRuntime::new(0.9, &Easing::CubicBezier(0.16, 1.0, 0.3, 1.0)));
static SPRING: LazyLock<AnimationRuntime> = LazyLock::new(|| {
    AnimationRuntime::new(3.0, &Easing::Spring { mass: 1.0, stiffness: 180.0, damping: 20.0 })
});

pub struct TantivyTurboDemoVideo<'a> {
    pub media: &'a TantivyTurboDemoMedia,
    intro: Intro,
    upstream: VsUpstream,
    lucene_rs: VsLuceneRs,
    and_trick: AndTrick,
    exact: Exact,
    outro: Outro,
}

impl<'a> TantivyTurboDemoVideo<'a> {
    pub fn new(media: &'a TantivyTurboDemoMedia) -> Self {
        Self {
            media,
            intro: Intro,
            upstream: VsUpstream,
            lucene_rs: VsLuceneRs,
            and_trick: AndTrick,
            exact: Exact,
            outro: Outro,
        }
    }
}

impl std::fmt::Debug for TantivyTurboDemoVideo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TantivyTurboDemoVideo").finish()
    }
}

/// Overlap::Previous adds the cross-fade in front of a scene, so scenes simply add up.
fn total_seconds() -> f32 {
    SCENES.iter().sum::<f32>()
}

impl Video for TantivyTurboDemoVideo<'_> {
    const FPS: usize = 30;
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Auto
    }

    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }

    fn define_scenes(&self) -> Scenes<'_> {
        Scenes::from(vec![
            &self.intro as &dyn Scene,
            &self.upstream,
            &self.lucene_rs,
            &self.and_trick,
            &self.exact,
            &self.outro,
        ])
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let progress = (frame.seconds() / total_seconds()).clamp(0.0, 1.0);
        let bar = (progress * 1536.0).max(4.0);
        fframes::svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" width={WIDTH} height={HEIGHT} viewBox="0 0 1920 1080">
                <rect width="1920" height="1080" fill={BG} />
                {ctx.render_scenes(&frame)}
                <rect x="192" y="988" width="1536" height="4" rx="2" fill={LINE} />
                <rect x="192" y="988" width={bar} height="4" rx="2" fill={BLUE} />
                <text x="192" y="1040" font-family={MONO} font-weight={W_MONO} font-size="26" fill={MUTED}>"tantivy-turbo"</text>
            </svg>
        )
    }
}

/// 0 to 1 over 0.8 s from `start` (seconds into the scene).
fn ramp(frame: &Frame, start: f32) -> f32 {
    frame.animate_runtime(AnimateRuntimeInput { on_second: start, from: 0.0, to: 1.0, animation_runtime: &EASE })
}

/// 0 to 1 over 0.9 s with a quick start and a soft landing.
fn grow(frame: &Frame, start: f32) -> f32 {
    frame.animate_runtime(AnimateRuntimeInput { on_second: start, from: 0.0, to: 1.0, animation_runtime: &GROW })
}

fn rise(frame: &Frame, start: f32) -> f32 {
    frame.animate_runtime(AnimateRuntimeInput { on_second: start, from: 40.0, to: 0.0, animation_runtime: &SPRING })
}

/// Fade through the background during the overlap, keeping titles legible.
fn scene_alpha(frame: &Frame, length: f32) -> f32 {
    let t = frame.seconds();
    ((t - XFADE / 2.0) / (XFADE / 2.0))
        .min((length + XFADE / 2.0 - t) / (XFADE / 2.0))
        .clamp(0.0, 1.0)
}

/// A horizontal bar growing right from `x`: square at the baseline, 8px rounded data end.
fn bar<'a>(x: f32, y: f32, width: f32, height: f32, fill: &'a str) -> Svgr<'a> {
    if width < 1.0 {
        return Svgr::empty();
    }
    let r = 8.0_f32.min(width / 2.0).min(height / 2.0);
    let d = format!(
        "M{x},{y} H{} Q{},{y} {},{} V{} Q{},{} {},{} H{x} Z",
        x + width - r,
        x + width,
        x + width,
        y + r,
        y + height - r,
        x + width,
        y + height,
        x + width - r,
        y + height
    );
    fframes::svgr!(<path d={d} fill={fill} />)
}

fn heading<'a>(kicker: &'a str, title: &'a str, frame: &Frame) -> Svgr<'a> {
    fframes::svgr!(<g opacity={ramp(frame, 0.1)} transform={Transform::translate(0.0, rise(frame, 0.1))}>
        <rect x="192" y="152" width="48" height="6" rx="3" fill={BLUE} />
        <text x="256" y="162" font-family={MONO} font-weight={W_MONO} font-size="40" letter-spacing="1" fill={INK2}>{kicker}</text>
        <text x="186" y="262" font-family={SANS} font-weight={W_SANS} font-size="88" letter-spacing="-3" fill={INK}>{title}</text>
    </g>)
}

#[derive(Debug)]
struct Intro;

impl Scene for Intro {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[0])
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        // Posting-list blocks light up left to right, then settle behind the title.
        let tiles: Vec<Svgr> = (0..24)
            .map(|i| {
                let front = (t * 9.0 - i as f32).clamp(0.0, 1.0);
                if front == 0.0 {
                    return Svgr::empty();
                }
                let settle = 1.0 - 0.65 * ramp(&frame, 2.0);
                let color = if i % 5 == 3 { AQUA } else { BLUE };
                let x = 192.0 + i as f32 * 64.0;
                let h = 40.0 + 70.0 * (((i * 7) % 9) as f32 / 8.0);
                fframes::svgr!(<rect x={x} y={300.0 - h * front} width="52" height={h * front} rx="8" fill={color} opacity={front * settle} />)
            })
            .collect();
        let alpha = (t / 0.2).min((SCENES[0] - XFADE / 2.0 - t) / 0.2).clamp(0.0, 1.0);
        let title = ramp(&frame, 0.35);
        let sub = ramp(&frame, 0.9);
        fframes::svgr!(<g opacity={alpha}>
            {tiles}
            <g opacity={title} transform={Transform::translate(0.0, rise(&frame, 0.35))}>
                <text x="186" y="520" font-family={SANS} font-weight={W_SANS} font-size="170" letter-spacing="-6" fill={INK}>"Search. "<tspan fill={BLUE}>"Faster."</tspan></text>
            </g>
            <g opacity={sub} transform={Transform::translate(0.0, rise(&frame, 0.9))}>
                <text x="192" y="625" font-family={SANS} font-weight={W_SANS} font-size="48" fill={INK2}>"Find the best matches in a collection."</text>
                <text x="192" y="700" font-family={SANS} font-weight={W_SANS} font-size="48" fill={INK2}>"Faster search. Results checked."</text>
                <text x="192" y="775" font-family={SANS} font-weight={W_SANS} font-size="40" fill={MUTED}>"BEST 10: top matches  ·  COUNT: number of matches"</text>
            </g>
        </g>)
    }
}

#[derive(Debug)]
struct VsUpstream;

impl Scene for VsUpstream {
    fn duration(&self) -> Duration<'_> { Duration::Seconds(SCENES[1]) }
    fn overlap(&self) -> Overlap { Overlap::Previous(XFADE) }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let sweep = grow(&frame, 0.45);
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[1])}>
            {heading("VS TANTIVY 0.27 · 1,201 SEARCHES", "Same searches. Less time.", &frame)}
            <g opacity={ramp(&frame, 0.4)} transform={Transform::translate(0.0, rise(&frame, 0.4))}>
                <text x="192" y="510" font-family={SANS} font-weight={W_SANS} font-size="188" letter-spacing="-6" fill={BLUE}>"1.79×"</text>
                <text x="192" y="600" font-family={SANS} font-weight={W_SANS} font-size="60" fill={INK}>"faster to find the best 10 results"</text>
                {bar(192.0, 640.0, 1120.0 * sweep, 12.0, BLUE)}
            </g>
            <text x="192" y="775" font-family={SANS} font-weight={W_SANS} font-size="52" fill={INK2} opacity={ramp(&frame, 1.25)}><tspan fill={AQUA}>"1.59×"</tspan>" faster to count all matches"</text>
            <text x="192" y="940" font-family={MONO} font-weight={W_MONO} font-size="40" fill={INK2} opacity={ramp(&frame, 1.25)}>"Historical · Oct 23 · before skip-reader fix"</text>
        </g>)
    }
}

#[derive(Debug)]
struct VsLuceneRs;

impl Scene for VsLuceneRs {
    fn duration(&self) -> Duration<'_> { Duration::Seconds(SCENES[2]) }
    fn overlap(&self) -> Overlap { Overlap::Previous(XFADE) }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let sweep = grow(&frame, 0.45);
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[2])}>
            {heading("VS LUCENE-RS · 1,201 SEARCHES", "Against another Rust library", &frame)}
            <g opacity={ramp(&frame, 0.4)} transform={Transform::translate(0.0, rise(&frame, 0.4))}>
                <text x="192" y="510" font-family={SANS} font-weight={W_SANS} font-size="188" letter-spacing="-6" fill={BLUE}>"1.25×"</text>
                <text x="192" y="600" font-family={SANS} font-weight={W_SANS} font-size="60" fill={INK}>"faster to find the best 10 results"</text>
                {bar(192.0, 640.0, 1000.0 * sweep, 12.0, BLUE)}
            </g>
            <text x="192" y="775" font-family={SANS} font-weight={W_SANS} font-size="50" fill={INK2} opacity={ramp(&frame, 1.25)}>"Match any of 3+ words: 1.04× faster"</text>
            <text x="192" y="940" font-family={MONO} font-weight={W_MONO} font-size="40" fill={INK2} opacity={ramp(&frame, 1.25)}>"Historical · Oct 23 · before skip-reader fix"</text>
        </g>)
    }
}

#[derive(Debug)]
struct AndTrick;

impl Scene for AndTrick {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[3])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        // A concrete, intentionally small illustration. The highlighted rows
        // are documents containing the rare word; the final check keeps both.
        let rare = ramp(&frame, 1.4);
        let both = ramp(&frame, 3.6);
        let rows = [
            ("A", "quokka", "animal", true, true),
            ("B", "—", "animal", false, false),
            ("C", "quokka", "animal", true, true),
            ("D", "—", "animal", false, false),
            ("E", "quokka", "—", true, false),
        ];
        let documents: Vec<Svgr> = rows.iter().enumerate().map(|(i, &(id, first, second, has_rare, has_both))| {
            let y = 405.0 + i as f32 * 92.0;
            let dim = if has_rare { 1.0 } else { 1.0 - 0.48 * rare };
            let keep = if has_both { 1.0 } else { 1.0 - 0.48 * both };
            let highlight = if has_rare { rare * keep } else { 0.0 };
            fframes::svgr!(<g opacity={ramp(&frame, 0.35 + i as f32 * 0.08)}>
                <rect x="192" y={y - 52.0} width="1536" height="76" rx="14" fill={PANEL} />
                <rect x="192" y={y - 52.0} width="8" height="76" rx="4" fill={BLUE} opacity={highlight} />
                <g opacity={dim * keep}>
                    <text x="224" y={y} font-family={MONO} font-weight={W_MONO} font-size="34" fill={INK2}>{id}</text>
                    <text x="455" y={y} font-family={SANS} font-weight={W_SANS} font-size="44" fill={if has_rare { INK } else { MUTED }}>{first}</text>
                    <text x="940" y={y} font-family={SANS} font-weight={W_SANS} font-size="44" fill={INK2}>{second}</text>
                </g>
                <text x="1650" y={y} text-anchor="end" font-family={SANS} font-weight={W_SANS} font-size="36" fill={AQUA} opacity={if has_both { both } else { 0.0 }}>"MATCH"</text>
            </g>)
        }).collect();
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[3])}>
            {heading("HOW IT WORKS · ILLUSTRATIVE EXAMPLE", "Find both words", &frame)}
            <text x="192" y="326" font-family={SANS} font-weight={W_SANS} font-size="38" fill={INK2}>"Search: "<tspan fill={BLUE}>"quokka"</tspan>" AND animal"</text>
            {documents}
            <text x="192" y="895" font-family={SANS} font-weight={W_SANS} font-size="44" fill={INK} opacity={ramp(&frame, 1.4)}>"1. Start with the rare word: fewer documents to check."</text>
            <text x="192" y="955" font-family={SANS} font-weight={W_SANS} font-size="44" fill={INK2} opacity={ramp(&frame, 3.6)}>"2. Keep the documents that also contain animal."</text>
        </g>)
    }
}

#[derive(Debug)]
struct Exact;

impl Scene for Exact {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[4])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let rows = [
            ("Results match the previous build", "3 × 1,227 searches"),
            ("Checked against a full scan", "4,800 random checks"),
            ("Compared with the Lucene Rust port", "1,227 searches"),
        ];
        let items: Vec<Svgr> = rows
            .iter()
            .enumerate()
            .map(|(i, &(what, value))| {
                let start = 0.5 + i as f32 * 0.25;
                let y = 420.0 + i as f32 * 150.0;
                let draw = ramp(&frame, start + 0.15);
                fframes::svgr!(<g opacity={ramp(&frame, start)} transform={Transform::translate(0.0, rise(&frame, start))}>
                    <rect x="192" y={y - 62.0} width="88" height="88" rx="22" fill={PANEL} />
                    <path d={format!("M{} {} l18 18 l38 -42", 214.0, y - 16.0)} stroke={AQUA} stroke-width="8" fill="none"
                        stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="90" stroke-dashoffset={90.0 * (1.0 - draw)} />
                    <text x="320" y={y} font-family={SANS} font-weight={W_SANS} font-size="46" fill={INK}>{what}</text>
                    <text x="1728" y={y} text-anchor="end" font-family={MONO} font-weight={W_MONO} font-size="44" fill={INK2}>{value}</text>
                </g>)
            })
            .collect();
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[4])}>
            {heading("CORRECTNESS", "Results checked", &frame)}
            {items}
        </g>)
    }
}

#[derive(Debug)]
struct Outro;

impl Scene for Outro {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[5])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let card = ramp(&frame, 0.3);
        fframes::svgr!(<g opacity={(t / XFADE).clamp(0.0, 1.0)}>
            <g opacity={card} transform={Transform::translate(0.0, rise(&frame, 0.3))}>
                <text x="192" y="450" font-family={SANS} font-weight={W_SANS} font-size="108" fill={INK}>"Fast search."</text>
                <text x="192" y="585" font-family={SANS} font-weight={W_SANS} font-size="108" fill={BLUE}>"Checked answers."</text>
                <text x="192" y="700" font-family={SANS} font-weight={W_SANS} font-size="52" fill={INK2}>"tantivy-turbo · a Rust search library"</text>
                <text x="192" y="800" font-family={MONO} font-weight={W_MONO} font-size="34" fill={MUTED}>"github.com/Priyansh4444/tantivy-turbo"</text>
            </g>
        </g>)
    }
}
