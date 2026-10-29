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
        <text x="186" y="262" font-family={SANS} font-weight={W_SANS} font-size="72" letter-spacing="-2" fill={INK}>{title}</text>
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
        let matched = ramp(&frame, 2.0);
        let documents: Vec<Svgr> = (0..4).map(|i| {
            let start = 0.45 + i as f32 * 0.16;
            let p = grow(&frame, start);
            let x = 850.0 + i as f32 * 195.0;
            let y = 430.0 + (1.0 - p) * 58.0;
            let hit = i == 0 || i == 2;
            fframes::svgr!(<g opacity={ramp(&frame, start)}>
                <rect x={x} y={y} width="158" height="214" rx="16" fill={PANEL} stroke={LINE} stroke-width="2" />
                <rect x={x} y={y} width="158" height="214" rx="16" fill="none" stroke={BLUE} stroke-width="4" opacity={if hit { matched } else { 0.0 }} />
                <line x1={x + 24.0} y1={y + 56.0} x2={x + 128.0} y2={y + 56.0} stroke={INK2} stroke-width="6" stroke-linecap="round" />
                <line x1={x + 24.0} y1={y + 90.0} x2={x + 113.0} y2={y + 90.0} stroke={MUTED} stroke-width="5" stroke-linecap="round" />
                <line x1={x + 24.0} y1={y + 122.0} x2={x + 120.0} y2={y + 122.0} stroke={MUTED} stroke-width="5" stroke-linecap="round" />
                <circle cx={x + 124.0} cy={y + 180.0} r="14" fill={AQUA} opacity={if hit { matched } else { 0.0 }} />
            </g>)
        }).collect();
        let alpha = (t / 0.2).min((SCENES[0] - XFADE / 2.0 - t) / 0.2).clamp(0.0, 1.0);
        let sweep = grow(&frame, 1.2);
        fframes::svgr!(<g opacity={alpha}>
            <text x="192" y="170" font-family={MONO} font-weight={W_MONO} font-size="34" fill={MUTED} opacity={ramp(&frame, 0.1)}>"TANTIVY-TURBO · RUST SEARCH"</text>
            <text x="192" y="300" font-family={SANS} font-weight={W_SANS} font-size="100" fill={INK} opacity={ramp(&frame, 0.2)}>"Find what matters."</text>
            <g opacity={ramp(&frame, 0.45)}>
                <rect x="192" y="475" width="550" height="110" rx="22" fill={PANEL} stroke={LINE} stroke-width="2" />
                <circle cx="244" cy="526" r="16" fill="none" stroke={BLUE} stroke-width="6" />
                <line x1="256" y1="538" x2="269" y2="551" stroke={BLUE} stroke-width="6" stroke-linecap="round" />
                <text x="300" y="544" font-family={SANS} font-weight={W_SANS} font-size="48" fill={INK}>"quokka animal"</text>
                <line x1="760" y1="530" x2={760.0 + 68.0 * sweep} y2="530" stroke={BLUE} stroke-width="6" stroke-linecap="round" />
            </g>
            {documents}
            <text x="192" y="790" font-family={SANS} font-weight={W_SANS} font-size="52" fill={INK2} opacity={ramp(&frame, 1.0)}>"Search a collection. Return the best matches."</text>
            <text x="1728" y="930" text-anchor="end" font-family={MONO} font-weight={W_MONO} font-size="26" fill={MUTED} opacity={ramp(&frame, 1.0)}>"illustrative"</text>
        </g>)
    }
}

#[derive(Debug)]
struct VsUpstream;

impl Scene for VsUpstream {
    fn duration(&self) -> Duration<'_> { Duration::Seconds(SCENES[1]) }
    fn overlap(&self) -> Overlap { Overlap::Previous(XFADE) }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let baseline = grow(&frame, 0.55);
        let turbo = grow(&frame, 0.95);
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[1])}>
            {heading("VS TANTIVY 0.27 · 1,201 SEARCHES", "Finding the best 10", &frame)}
            <text x="192" y="385" font-family={SANS} font-weight={W_SANS} font-size="102" fill={BLUE} opacity={ramp(&frame, 0.3)}>"1.79× faster"</text>
            <text x="192" y="475" font-family={SANS} font-weight={W_SANS} font-size="38" fill={INK2} opacity={ramp(&frame, 0.4)}>"Relative search time · less is better"</text>
            <text x="192" y="565" font-family={SANS} font-weight={W_SANS} font-size="42" fill={INK2} opacity={ramp(&frame, 0.55)}>"Tantivy 0.27"</text>
            {bar(620.0, 522.0, 900.0 * baseline, 56.0, MUTED)}
            <text x="1540" y="565" font-family={MONO} font-weight={W_MONO} font-size="36" fill={INK2} opacity={ramp(&frame, 1.2)}>"1.00×"</text>
            <text x="192" y="690" font-family={SANS} font-weight={W_SANS} font-size="42" fill={INK} opacity={ramp(&frame, 0.95)}>"tantivy-turbo"</text>
            {bar(620.0, 647.0, (900.0 / 1.79) * turbo, 56.0, BLUE)}
            <text x="1540" y="690" font-family={MONO} font-weight={W_MONO} font-size="36" fill={INK} opacity={ramp(&frame, 1.6)}>"~0.56×"</text>
            <text x="192" y="805" font-family={SANS} font-weight={W_SANS} font-size="44" fill={INK2} opacity={ramp(&frame, 1.4)}><tspan fill={AQUA}>"1.59×"</tspan>" faster to count all matches"</text>
            <text x="192" y="940" font-family={MONO} font-weight={W_MONO} font-size="36" fill={INK2} opacity={ramp(&frame, 1.25)}>"Historical · Oct 23 · before skip-reader fix"</text>
        </g>)
    }
}

#[derive(Debug)]
struct VsLuceneRs;

impl Scene for VsLuceneRs {
    fn duration(&self) -> Duration<'_> { Duration::Seconds(SCENES[2]) }
    fn overlap(&self) -> Overlap { Overlap::Previous(XFADE) }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let baseline = grow(&frame, 0.55);
        let turbo = grow(&frame, 0.95);
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[2])}>
            {heading("VS LUCENE-RS · 1,201 SEARCHES", "Finding the best 10", &frame)}
            <text x="192" y="385" font-family={SANS} font-weight={W_SANS} font-size="102" fill={BLUE} opacity={ramp(&frame, 0.3)}>"1.25× faster"</text>
            <text x="192" y="475" font-family={SANS} font-weight={W_SANS} font-size="38" fill={INK2} opacity={ramp(&frame, 0.4)}>"Relative search time · less is better"</text>
            <text x="192" y="565" font-family={SANS} font-weight={W_SANS} font-size="42" fill={INK2} opacity={ramp(&frame, 0.55)}>"lucene-rs"</text>
            {bar(620.0, 522.0, 900.0 * baseline, 56.0, MUTED)}
            <text x="1540" y="565" font-family={MONO} font-weight={W_MONO} font-size="36" fill={INK2} opacity={ramp(&frame, 1.2)}>"1.00×"</text>
            <text x="192" y="690" font-family={SANS} font-weight={W_SANS} font-size="42" fill={INK} opacity={ramp(&frame, 0.95)}>"tantivy-turbo"</text>
            {bar(620.0, 647.0, (900.0 / 1.25) * turbo, 56.0, BLUE)}
            <text x="1540" y="690" font-family={MONO} font-weight={W_MONO} font-size="36" fill={INK} opacity={ramp(&frame, 1.6)}>"~0.80×"</text>
            <text x="192" y="805" font-family={SANS} font-weight={W_SANS} font-size="44" fill={INK2} opacity={ramp(&frame, 1.4)}>"Match any of 3+ words: 1.04× faster"</text>
            <text x="192" y="940" font-family={MONO} font-weight={W_MONO} font-size="36" fill={INK2} opacity={ramp(&frame, 1.25)}>"Historical · Oct 23 · before skip-reader fix"</text>
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
        // Illustrative documents: A and C contain both words, E only the rare word.
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
            let x = 192.0 + i as f32 * 312.0;
            let lift = if has_both { 22.0 * both } else { 0.0 };
            let dim = if has_rare { 1.0 } else { 1.0 - 0.38 * rare };
            let keep = if has_both { 1.0 } else { 1.0 - 0.38 * both };
            let highlight = if has_rare { rare * if has_both { 1.0 } else { 1.0 - both } } else { 0.0 };
            fframes::svgr!(<g opacity={ramp(&frame, 0.3 + i as f32 * 0.1)} transform={Transform::translate(0.0, -lift)}>
                <rect x={x} y="395" width="280" height="324" rx="20" fill={PANEL} stroke={LINE} stroke-width="2" />
                <rect x={x} y="395" width="280" height="324" rx="20" fill="none" stroke={BLUE} stroke-width="4" opacity={highlight} />
                <g opacity={dim * keep}>
                    <rect x={x + 24.0} y="424" width="36" height="44" rx="4" fill="none" stroke={MUTED} stroke-width="3" />
                    <line x1={x + 32.0} y1="439" x2={x + 52.0} y2="439" stroke={MUTED} stroke-width="3" />
                    <line x1={x + 32.0} y1="450" x2={x + 52.0} y2="450" stroke={MUTED} stroke-width="3" />
                    <text x={x + 75.0} y="456" font-family={MONO} font-weight={W_MONO} font-size="30" fill={INK2}>{format!("DOC {id}")}</text>
                    <text x={x + 24.0} y="540" font-family={SANS} font-weight={W_SANS} font-size="42" fill={if has_rare { INK } else { MUTED }}>{first}</text>
                    <text x={x + 24.0} y="604" font-family={SANS} font-weight={W_SANS} font-size="42" fill={INK2}>{second}</text>
                </g>
                <text x={x + 24.0} y="678" font-family={MONO} font-weight={W_MONO} font-size="30" fill={AQUA} opacity={if has_both { both } else { 0.0 }}>"MATCH"</text>
            </g>)
        }).collect();
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[3])}>
            {heading("ILLUSTRATIVE SEARCH", "Find both words", &frame)}
            <text x="192" y="326" font-family={SANS} font-weight={W_SANS} font-size="42" fill={INK2}>"quokka "<tspan fill={BLUE}>"AND"</tspan>" animal"</text>
            {documents}
            <text x="192" y="850" font-family={SANS} font-weight={W_SANS} font-size="44" fill={INK} opacity={ramp(&frame, 1.4)}>"1  Find the rare word."</text>
            <text x="192" y="920" font-family={SANS} font-weight={W_SANS} font-size="44" fill={INK2} opacity={ramp(&frame, 3.6)}>"2  Keep documents with both words."</text>
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
            ("Results match lucene-rs", "1,227 searches"),
            ("Checked against a full scan", "14,400 random checks"),
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
