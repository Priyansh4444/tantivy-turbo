//! tantivy-turbo launch video: eight short scenes rendered with fframes.
//!
//! Benchmark figures come from the repository (`bench-results/` and
//! `doc/performance/`); the AND window count is an illustrative diagram.
//! The look matches the README charts:
//! solid rounded bars, blue for TOP-10 and cyan for COUNT, and cool ink on a
//! deep navy surface.
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

const BG: &str = "#090f1a";
const PANEL: &str = "#141f30";
const LINE: &str = "#29394d";
const AXIS: &str = "#75899e";
const INK: &str = "#edf3ff";
const INK2: &str = "#bbc9dc";
const MUTED: &str = "#8e9fb4";
const BLUE: &str = "#79a8ff";
const AQUA: &str = "#4ec8d7";
const SANS: &str = "DM Sans";
const MONO: &str = "JetBrains Mono";
const W_SANS: u16 = 500;
const W_MONO: u16 = 400;

/// Seconds shared by adjacent scenes. Titles fade through the background so
/// two explanations are never superimposed during the transition.
const XFADE: f32 = 0.4;
const SCENES: [f32; 8] = [4.0, 9.0, 8.0, 7.0, 7.5, 8.0, 6.0, 5.0];

static EASE: LazyLock<AnimationRuntime> =
    LazyLock::new(|| AnimationRuntime::new(0.8, &Easing::CubicBezier(0.16, 1.0, 0.3, 1.0)));
static GROW: LazyLock<AnimationRuntime> =
    LazyLock::new(|| AnimationRuntime::new(1.1, &Easing::CubicBezier(0.65, 0.0, 0.35, 1.0)));
static SPRING: LazyLock<AnimationRuntime> = LazyLock::new(|| {
    AnimationRuntime::new(3.0, &Easing::Spring { mass: 1.0, stiffness: 180.0, damping: 20.0 })
});

pub struct TantivyTurboDemoVideo<'a> {
    pub media: &'a TantivyTurboDemoMedia,
    intro: Intro,
    upstream: VsUpstream,
    lucene_rs: VsLuceneRs,
    lucene: VsLucene,
    or_trick: OrTrick,
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
            lucene: VsLucene,
            or_trick: OrTrick,
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
            &self.lucene,
            &self.or_trick,
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
                <text x="1728" y="1040" text-anchor="end" font-family={MONO} font-weight={W_MONO} font-size="26" fill={MUTED}>"made with fframes"</text>
            </svg>
        )
    }
}

/// 0 to 1 over 0.8 s from `start` (seconds into the scene).
fn ramp(frame: &Frame, start: f32) -> f32 {
    frame.animate_runtime(AnimateRuntimeInput { on_second: start, from: 0.0, to: 1.0, animation_runtime: &EASE })
}

/// 0 to 1 over 1.1 s with ease-in-out, for growing bars.
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
        <text x="256" y="162" font-family={MONO} font-weight={W_MONO} font-size="28" letter-spacing="2" fill={MUTED}>{kicker}</text>
        <text x="186" y="262" font-family={SANS} font-weight={W_SANS} font-size="88" letter-spacing="-3" fill={INK}>{title}</text>
    </g>)
}

fn legend<'a>(x: f32, y: f32, frame: &Frame, start: f32) -> Svgr<'a> {
    fframes::svgr!(<g opacity={ramp(frame, start)}>
        <rect x={x} y={y - 20.0} width="26" height="22" rx="5" fill={BLUE} />
        <text x={x + 38.0} y={y} font-family={SANS} font-weight={W_SANS} font-size="30" fill={INK2}>"TOP-10"</text>
        <rect x={x + 180.0} y={y - 20.0} width="26" height="22" rx="5" fill={AQUA} />
        <text x={x + 218.0} y={y} font-family={SANS} font-weight={W_SANS} font-size="30" fill={INK2}>"COUNT"</text>
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
        let title = ramp(&frame, 1.0);
        let sub = ramp(&frame, 1.5);
        fframes::svgr!(<g opacity={alpha}>
            {tiles}
            <g opacity={title} transform={Transform::translate(0.0, rise(&frame, 1.0))}>
                <text x="186" y="560" font-family={SANS} font-weight={W_SANS} font-size="184" letter-spacing="-7" fill={INK}>"tantivy-turbo"</text>
            </g>
            <g opacity={sub} transform={Transform::translate(0.0, rise(&frame, 1.5))}>
                <text x="192" y="660" font-family={SANS} font-weight={W_SANS} font-size="50" fill={INK2}>"A faster Tantivy fork, with results checked against exhaustive scoring."</text>
            </g>
        </g>)
    }
}

#[derive(Debug)]
struct VsUpstream;

impl Scene for VsUpstream {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[1])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        // Speedup over Tantivy 0.27, summed time, mean of both run orders (TOP-10, COUNT).
        let rows = [
            ("OR, 3+ terms", 3.27_f32, 3.85_f32),
            ("OR, 2 terms", 2.31, 1.72),
            ("Phrase", 1.37, 1.07),
            ("Single term", 1.18, 1.06),
            ("AND", 1.35, 1.48),
            ("All 1,201 queries", 1.79, 1.59),
        ];
        // Log scale from 1x (same speed) to 4x.
        let scale = |v: f32| v.log2() / 4.0_f32.log2() * 980.0;
        let groups: Vec<Svgr> = rows
            .iter()
            .enumerate()
            .map(|(i, &(name, top10, count))| {
                let start = 0.7 + i as f32 * 0.1;
                let p = grow(&frame, start);
                let y = 352.0 + i as f32 * 96.0;
                let w1 = scale(top10) * p;
                let w2 = scale(count) * p;
                let weight = if i == 5 { INK } else { INK2 };
                fframes::svgr!(<g opacity={ramp(&frame, start)}>
                    <text x="192" y={y + 46.0} font-family={SANS} font-weight={W_SANS} font-size="38" fill={weight}>{name}</text>
                    {bar(620.0, y + 8.0, w1, 32.0, BLUE)}
                    {bar(620.0, y + 46.0, w2, 32.0, AQUA)}
                    <text x={636.0 + w1} y={y + 34.0} font-family={MONO} font-weight={W_MONO} font-size="28" fill={INK} opacity={ramp(&frame, start + 0.6)}>{format!("{top10:.2}×")}</text>
                    <text x={636.0 + w2} y={y + 72.0} font-family={MONO} font-weight={W_MONO} font-size="28" fill={INK2} opacity={ramp(&frame, start + 0.6)}>{format!("{count:.2}×")}</text>
                </g>)
            })
            .collect();
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[1])}>
            {heading("VS TANTIVY 0.27", "Speedup by query type", &frame)}
            {legend(1290.0, 300.0, &frame, 0.5)}
            <line x1="620" y1="340" x2="620" y2="930" stroke={AXIS} stroke-width="2" opacity={ramp(&frame, 0.5)} />
            <text x="620" y="326" font-family={MONO} font-weight={W_MONO} font-size="24" fill={INK2} opacity={ramp(&frame, 0.5)}>"1× same speed"</text>
            <text x="1110" y="326" text-anchor="middle" font-family={MONO} font-weight={W_MONO} font-size="24" fill={MUTED} opacity={ramp(&frame, 0.5)}>"2×"</text>
            <text x="1600" y="326" text-anchor="middle" font-family={MONO} font-weight={W_MONO} font-size="24" fill={MUTED} opacity={ramp(&frame, 0.5)}>"4×"</text>
            {groups}
            <text x="192" y="956" font-family={MONO} font-weight={W_MONO} font-size="24" fill={MUTED} opacity={ramp(&frame, 1.8)}>"Oct 23 run · geometric mean of both orders · before later skip-reader fix"</text>
        </g>)
    }
}

#[derive(Debug)]
struct VsLuceneRs;

impl Scene for VsLuceneRs {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[2])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        // TOP-10 speedup over lucene-rs (Rust port of Lucene 10), summed time.
        let rows = [
            ("Single term", 1.85_f32),
            ("AND", 1.58),
            ("Phrase", 1.24),
            ("OR, 3+ terms", 1.04),
            ("OR, 2 terms", 1.02),
            ("All 1,201 queries", 1.25),
        ];
        let scale = |v: f32| v.log2() / 2.0_f32.log2() * 860.0;
        let bars: Vec<Svgr> = rows
            .iter()
            .enumerate()
            .map(|(i, &(name, speedup))| {
                let start = 0.6 + i as f32 * 0.1;
                let y = 362.0 + i as f32 * 82.0;
                let w = scale(speedup) * grow(&frame, start);
                let weight = if i == 5 { INK } else { INK2 };
                fframes::svgr!(<g opacity={ramp(&frame, start)}>
                    <text x="192" y={y + 38.0} font-family={SANS} font-weight={W_SANS} font-size="38" fill={weight}>{name}</text>
                    {bar(620.0, y + 8.0, w, 40.0, BLUE)}
                    <text x={638.0 + w} y={y + 40.0} font-family={MONO} font-weight={W_MONO} font-size="30" fill={INK} opacity={ramp(&frame, start + 0.6)}>{format!("{speedup:.2}×")}</text>
                </g>)
            })
            .collect();
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[2])}>
            {heading("VS LUCENE-RS · RUST PORT OF LUCENE 10", "1.25× faster, same bits", &frame)}
            <line x1="620" y1="350" x2="620" y2="860" stroke={AXIS} stroke-width="2" opacity={ramp(&frame, 0.5)} />
            <text x="620" y="336" font-family={MONO} font-weight={W_MONO} font-size="24" fill={INK2} opacity={ramp(&frame, 0.5)}>"1× same speed"</text>
            <text x="1123" y="336" text-anchor="middle" font-family={MONO} font-weight={W_MONO} font-size="24" fill={MUTED} opacity={ramp(&frame, 0.5)}>"1.5×"</text>
            <text x="1480" y="336" text-anchor="middle" font-family={MONO} font-weight={W_MONO} font-size="24" fill={MUTED} opacity={ramp(&frame, 0.5)}>"2×"</text>
            {bars}
            <g opacity={ramp(&frame, 2.0)}>
                <rect x="1390" y="540" width="338" height="180" rx="20" fill={PANEL} />
                <text x="1420" y="630" font-family={SANS} font-weight={W_SANS} font-size="84" letter-spacing="-2" fill={INK}>"7.1×"</text>
                <text x="1424" y="684" font-family={SANS} font-weight={W_SANS} font-size="30" fill={INK2}>"faster COUNT"</text>
            </g>
            <text x="192" y="914" font-family={MONO} font-weight={W_MONO} font-size="24" fill={MUTED} opacity={ramp(&frame, 1.8)}>"Oct 23 benchmark · before later skip-reader fix"</text>
            <text x="192" y="956" font-family={MONO} font-weight={W_MONO} font-size="24" fill={MUTED} opacity={ramp(&frame, 1.8)}>"identical top-10 doc IDs, score bits and counts on 1,227 checked queries"</text>
        </g>)
    }
}

#[derive(Debug)]
struct VsLucene;

impl Scene for VsLucene {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[3])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let stats = [
            ("1.8–2.1×", "faster TOP-10", BLUE),
            ("1.3×", "faster COUNT", AQUA),
            ("20 / 20", "queries faster", INK2),
        ];
        let cards: Vec<Svgr> = stats
            .iter()
            .enumerate()
            .map(|(i, &(value, what, accent))| {
                let start = 0.5 + i as f32 * 0.15;
                let x = 192.0 + i as f32 * 524.0;
                fframes::svgr!(<g opacity={ramp(&frame, start)} transform={Transform::translate(0.0, rise(&frame, start))}>
                    <rect x={x} y="350" width="488" height="300" rx="24" fill={PANEL} />
                    <rect x={x + 40.0} y="392" width="56" height="6" rx="3" fill={accent} />
                    <text x={x + 36.0} y="530" font-family={SANS} font-weight={W_SANS} font-size={if i == 0 { 88 } else { 116 }} letter-spacing="-4" fill={INK}>{value}</text>
                    <text x={x + 40.0} y="600" font-family={SANS} font-weight={W_SANS} font-size="40" fill={INK2}>{what}</text>
                </g>)
            })
            .collect();
        // Memory: tantivy-turbo vs Lucene with the default JVM heap, warmed process RSS.
        let p = grow(&frame, 1.5);
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[3])}>
            {heading("VS APACHE LUCENE 10.4 · EARLIER FORK BUILD", "Measured against Lucene", &frame)}
            {cards}
            <g opacity={ramp(&frame, 1.4)}>
                <text x="192" y="752" font-family={SANS} font-weight={W_SANS} font-size="32" fill={INK2}>"resident memory"</text>
                {bar(500.0, 724.0, 1228.0 * 0.86 * 14.9 / 346.3 * p, 36.0, BLUE)}
                <text x={516.0 + 1228.0 * 0.86 * 14.9 / 346.3 * p} y="752" font-family={MONO} font-weight={W_MONO} font-size="28" fill={INK}>"15 MiB fork"</text>
                {bar(500.0, 790.0, 1228.0 * p * 0.86, 36.0, AXIS)}
                <text x={516.0 + 1228.0 * p * 0.86} y="818" font-family={MONO} font-weight={W_MONO} font-size="28" fill={INK2}>"346 MiB Lucene"</text>
            </g>
            <text x="192" y="956" font-family={MONO} font-weight={W_MONO} font-size="24" fill={MUTED} opacity={ramp(&frame, 1.8)}>"2026-10-04 · geometric means · 20 queries · default JVM heap"</text>
        </g>)
    }
}

#[derive(Debug)]
struct OrTrick;

impl Scene for OrTrick {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[4])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        // Regions on one 21-term query: many thin slivers merge into a few wide regions.
        let merge = grow(&frame, 1.6);
        let scan = grow(&frame, 0.7);
        // These are measured endpoints, not measurements during the animation.
        let count = if merge < 0.5 { 30_242 } else { 362 };
        let thin = 120;
        let show = ramp(&frame, 0.5);
        let slivers: Vec<Svgr> = (0..thin)
            .map(|i| {
                let w = 1536.0 / thin as f32;
                fframes::svgr!(<rect x={192.0 + i as f32 * w} y="430" width={w - 3.0} height="170" rx="2" fill={AXIS} opacity={(1.0 - merge) * show} />)
            })
            .collect();
        let wide = [0.0, 0.14, 0.27, 0.45, 0.58, 0.76, 0.88, 1.0];
        let blocks: Vec<Svgr> = wide
            .windows(2)
            .filter_map(|edge| {
                let x = 192.0 + edge[0] * 1536.0;
                let w = ((edge[1] - edge[0]) * 1536.0 - 8.0) * merge;
                if w < 1.0 { return None; }
                Some(fframes::svgr!(<rect x={x} y="430" width={w} height="170" rx="14" fill={BLUE} opacity={merge} />))
            })
            .collect();
        let digits = format_thousands(count);
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[4])}>
            {heading("HOW IT WORKS · OR", "Fewer, wider MaxScore regions", &frame)}
            <text x="192" y="390" font-family={MONO} font-weight={W_MONO} font-size="56" fill={INK} opacity={show}>{digits}</text>
            <text x="500" y="390" font-family={SANS} font-weight={W_SANS} font-size="38" fill={INK2} opacity={show}>"regions for one 21-term query"</text>
            <text x="1728" y="390" text-anchor="end" font-family={MONO} font-weight={W_MONO} font-size="26" fill={MUTED} opacity={show}>"schematic · not to scale"</text>
            {slivers}
            {blocks}
            <line x1={192.0 + 1536.0 * scan} y1="420" x2={192.0 + 1536.0 * scan} y2="610" stroke={AQUA} stroke-width="4" opacity={show * (1.0 - merge)} />
            <text x="192" y="720" font-family={SANS} font-weight={W_SANS} font-size="42" fill={INK} opacity={ramp(&frame, 2.6)}>"Bounds from cached range maxima instead of every block."</text>
            <text x="192" y="790" font-family={SANS} font-weight={W_SANS} font-size="42" fill={INK2} opacity={ramp(&frame, 2.9)}>"Candidates scored in Lucene-style bulk windows."</text>
        </g>)
    }
}

#[derive(Debug)]
struct AndTrick;

impl Scene for AndTrick {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[5])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let x0 = 420.0_f32;
        let span = 1308.0_f32;
        let rare = [0.07_f32, 0.31, 0.335, 0.62, 0.86];
        let rare_blocks = [0.2_f32, 0.5, 0.75];
        let show = ramp(&frame, 0.4);
        let morph = grow(&frame, 2.2);
        // Lanes: the rare term's postings and blocks, a common term's many blocks.
        let mut lanes: Vec<Svgr> = Vec::new();
        for &c in &rare_blocks {
            let x = x0 + c * span;
            lanes.push(fframes::svgr!(<line x1={x} y1="392" x2={x} y2="420" stroke={AXIS} stroke-width="3" />));
        }
        for (i, d) in rare.iter().enumerate() {
            let x = x0 + d * span;
            lanes.push(fframes::svgr!(<circle cx={x} cy="406" r="11" fill={INK} stroke={BG} stroke-width="4" opacity={ramp(&frame, 0.5 + i as f32 * 0.18)} />));
        }
        for i in 1..34 {
            let x = x0 + i as f32 / 34.0 * span;
            lanes.push(fframes::svgr!(<line x1={x} y1="474" x2={x} y2="502" stroke={AXIS} stroke-width="3" />));
        }
        // Windows: before, one per gap between any boundary; after, rare blocks from the next posting.
        let mut cuts: Vec<f32> = (1..34).map(|i| i as f32 / 34.0).chain(rare_blocks).collect();
        cuts.sort_by(|a, b| a.total_cmp(b));
        cuts.dedup_by(|a, b| (*a - *b).abs() < f32::EPSILON);
        let mut edges = vec![0.0_f32];
        edges.extend(cuts);
        edges.push(1.0);
        let before: Vec<Svgr> = edges
            .windows(2)
            .map(|e| {
                let x = x0 + e[0] * span + 2.0;
                let w = (e[1] - e[0]) * span - 4.0;
                fframes::svgr!(<rect x={x} y="560" width={w.max(1.0)} height="64" rx="6" fill={AXIS} opacity={1.0 - morph} />)
            })
            .collect();
        let redges = [0.0_f32, 0.2, 0.5, 0.75, 1.0];
        let after: Vec<Svgr> = redges
            .windows(2)
            .filter_map(|e| {
                let start = rare.iter().copied().find(|&d| d >= e[0] && d < e[1])?;
                let x = x0 + start * span - 12.0;
                let w = ((e[1] - start) * span + 6.0) * morph;
                if w < 1.0 { return None; }
                Some(fframes::svgr!(<rect x={x} y="560" width={w} height="64" rx="14" fill={BLUE} opacity={morph} />))
            })
            .collect();
        let windows = if morph < 0.5 { before.len() } else { after.len() }.to_string();
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[5])}>
            {heading("HOW IT WORKS · AND", "Windows follow the rarest term", &frame)}
            <g opacity={show}>
                <text x="192" y="418" font-family={SANS} font-weight={W_SANS} font-size="32" fill={INK2}>"rare term"</text>
                <line x1={x0} y1="406" x2={x0 + span} y2="406" stroke={LINE} stroke-width="3" />
                <text x="192" y="498" font-family={SANS} font-weight={W_SANS} font-size="32" fill={INK2}>"common term"</text>
                <line x1={x0} y1="488" x2={x0 + span} y2="488" stroke={LINE} stroke-width="3" />
                {lanes}
                <text x="192" y="604" font-family={SANS} font-weight={W_SANS} font-size="32" fill={INK}>"windows"</text>
                {before}
                {after}
                <text x="192" y="700" font-family={MONO} font-weight={W_MONO} font-size="52" fill={INK}>{windows}</text>
                <text x="290" y="700" font-family={SANS} font-weight={W_SANS} font-size="36" fill={INK2}>"windows to check"</text>
                <text x="1728" y="700" text-anchor="end" font-family={MONO} font-weight={W_MONO} font-size="26" fill={MUTED}>"illustrative example"</text>
            </g>
            <text x="192" y="800" font-family={SANS} font-weight={W_SANS} font-size="42" fill={INK} opacity={ramp(&frame, 3.2)}>"Windows end at the rare term's blocks and start at its next posting."</text>
            <text x="192" y="870" font-family={SANS} font-weight={W_SANS} font-size="42" fill={INK2} opacity={ramp(&frame, 3.5)}>"AND went from 1.2× slower than Tantivy to 1.35× faster."</text>
        </g>)
    }
}

fn format_thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[derive(Debug)]
struct Exact;

impl Scene for Exact {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[6])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let rows = [
            ("Top-10, scores & counts match prior build", "3 × 1,227 queries"),
            ("Pruned results equal exhaustive scoring", "4,800 random checks"),
            ("New unsafe code", "0 lines"),
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
                    <text x="1728" y={y} text-anchor="end" font-family={MONO} font-weight={W_MONO} font-size="38" fill={INK2}>{value}</text>
                </g>)
            })
            .collect();
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[6])}>
            {heading("CORRECTNESS", "Results checked", &frame)}
            {items}
        </g>)
    }
}

#[derive(Debug)]
struct Outro;

impl Scene for Outro {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[7])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let card = ramp(&frame, 0.3);
        fframes::svgr!(<g opacity={(t / XFADE).clamp(0.0, 1.0)}>
            <rect x="192" y="240" width="1536" height="420" rx="28" fill={PANEL} opacity={card} />
            <g opacity={card} transform={Transform::translate(0.0, rise(&frame, 0.3))} font-family={MONO} font-weight={W_MONO} font-size="44">
                <text x="250" y="350" fill={MUTED}>"[dependencies.tantivy]"</text>
                <text x="250" y="445" fill={INK}>"package = "<tspan fill={BLUE}>"\"tantivy-turbo\""</tspan></text>
                <text x="250" y="540" fill={INK}>"git = "<tspan fill={INK2}>"\"https://github.com/Priyansh4444/tantivy-turbo\""</tspan></text>
            </g>
            <g opacity={ramp(&frame, 1.0)} transform={Transform::translate(0.0, rise(&frame, 1.0))}>
                <text x="960" y="800" text-anchor="middle" font-family={SANS} font-weight={W_SANS} font-size="60" fill={INK}>"Drop-in. Your code keeps using tantivy::"</text>
            </g>
        </g>)
    }
}
