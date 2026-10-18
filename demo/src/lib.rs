//! tantivy-turbo launch video: six short scenes rendered with fframes.
//!
//! Every number on screen comes from the benchmarks in the repository README
//! (`bench/` and `doc/performance/`). Bars and backgrounds use ordered (Bayer)
//! dithering: 16 fill levels per color, defined once as SVG patterns, so a fill can
//! "dissolve" in by stepping through the levels.
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

const BG: &str = "#0c0e12";
const INK: &str = "#ecebe4";
const MUTED: &str = "#8b919c";
const LIME: &str = "#c6f35a";
const TEAL: &str = "#5cc8b8";
const SLATE: &str = "#5a606b";
const SANS: &str = "DM Sans";
const MONO: &str = "JetBrains Mono";
const W_SANS: u16 = 500;
const W_MONO: u16 = 400;

/// Seconds of cross-fade between scenes.
const XFADE: f32 = 0.4;
const SCENES: [f32; 6] = [4.0, 7.0, 5.5, 6.5, 4.5, 4.5];

static EASE: LazyLock<AnimationRuntime> =
    LazyLock::new(|| AnimationRuntime::new(0.8, &Easing::CubicBezier(0.16, 1.0, 0.3, 1.0)));
static GROW: LazyLock<AnimationRuntime> =
    LazyLock::new(|| AnimationRuntime::new(1.1, &Easing::CubicBezier(0.65, 0.0, 0.35, 1.0)));
static SPRING: LazyLock<AnimationRuntime> = LazyLock::new(|| {
    AnimationRuntime::new(3.0, &Easing::Spring { mass: 1.0, stiffness: 180.0, damping: 20.0 })
});

/// 4x4 Bayer thresholds: a cell is inked at level L when its value is below L.
const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

pub struct TantivyTurboDemoVideo<'a> {
    pub media: &'a TantivyTurboDemoMedia,
    intro: Intro,
    upstream: VsUpstream,
    lucene: VsLucene,
    trick: Trick,
    exact: Exact,
    outro: Outro,
}

impl<'a> TantivyTurboDemoVideo<'a> {
    pub fn new(media: &'a TantivyTurboDemoMedia) -> Self {
        Self { media, intro: Intro, upstream: VsUpstream, lucene: VsLucene, trick: Trick, exact: Exact, outro: Outro }
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
            &self.lucene,
            &self.trick,
            &self.exact,
            &self.outro,
        ])
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let progress = (frame.seconds() / total_seconds()).clamp(0.0, 1.0);
        let bar = (progress * 1536.0).max(0.5);
        fframes::svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" width={WIDTH} height={HEIGHT} viewBox="0 0 1920 1080">
                <defs>{dither_defs()}</defs>
                <rect width="1920" height="1080" fill={BG} />
                {ctx.render_scenes(&frame)}
                <rect x="192" y="985" width="1536" height="6" fill="url(#slate4)" />
                <rect x="192" y="985" width={bar} height="6" fill="url(#lime12)" />
                <text x="192" y="1040" font-family={MONO} font-weight={W_MONO} font-size="28" fill={MUTED}>"tantivy-turbo"</text>
                <text x="1728" y="1040" text-anchor="end" font-family={MONO} font-weight={W_MONO} font-size="28" fill={MUTED}>"made with fframes"</text>
            </svg>
        )
    }
}

/// 16 dither levels for each palette color, as 8x8 px tiles of 2x2 px cells.
fn dither_defs<'a>() -> Vec<Svgr<'a>> {
    let mut defs = Vec::new();
    for (name, color) in [("lime", LIME), ("teal", TEAL), ("slate", SLATE), ("ink", INK)] {
        for level in 1..=16u8 {
            let cells: Vec<Svgr> = (0..4)
                .flat_map(|y| (0..4).map(move |x| (x, y)))
                .filter(|&(x, y)| BAYER[y][x] < level)
                .map(|(x, y)| fframes::svgr!(<rect x={x * 2} y={y * 2} width="2" height="2" fill={color} />))
                .collect();
            defs.push(fframes::svgr!(
                <pattern id={format!("{name}{level}")} width="8" height="8" patternUnits="userSpaceOnUse">{cells}</pattern>
            ));
        }
    }
    defs
}

/// `url(#name<level>)` for a 0..=1 coverage; `None` when nothing would be inked.
fn dither(name: &str, coverage: f32) -> Option<String> {
    let level = (coverage.clamp(0.0, 1.0) * 16.0).round() as u8;
    (level > 0).then(|| format!("url(#{name}{level})"))
}

fn dithered_rect<'a>(name: &str, coverage: f32, x: f32, y: f32, w: f32, h: f32) -> Svgr<'a> {
    match dither(name, coverage) {
        Some(fill) if w >= 0.5 && h >= 0.5 => fframes::svgr!(<rect x={x} y={y} width={w} height={h} fill={fill} />),
        _ => Svgr::empty(),
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
    frame.animate_runtime(AnimateRuntimeInput { on_second: start, from: 48.0, to: 0.0, animation_runtime: &SPRING })
}

/// Opacity of a cross-faded scene of `length` seconds (plus the overlap in front of it):
/// fade in over the overlap, fade out over the last 0.3 s.
fn scene_alpha(frame: &Frame, length: f32) -> f32 {
    let t = frame.seconds();
    (t / XFADE).min((length + XFADE - t) / 0.3).clamp(0.0, 1.0)
}

fn heading<'a>(kicker: &'a str, title: &'a str, frame: &Frame) -> Svgr<'a> {
    fframes::svgr!(<g opacity={ramp(frame, 0.1)} transform={Transform::translate(0.0, rise(frame, 0.1))}>
        <text x="192" y="190" font-family={MONO} font-weight={W_MONO} font-size="30" fill={LIME}>{kicker}</text>
        <text x="186" y="290" font-family={SANS} font-weight={W_SANS} font-size="92" letter-spacing="-3" fill={INK}>{title}</text>
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
        // A dither wave sweeps diagonally across a 16x9 grid of tiles and settles to a faint texture.
        let tiles: Vec<Svgr> = (0..9)
            .flat_map(|row| (0..16).map(move |col| (col, row)))
            .map(|(col, row)| {
                let front = t * 16.0 - (col + row) as f32;
                let coverage = if front < 0.0 {
                    0.0
                } else if front < 3.0 {
                    front / 3.0
                } else {
                    (1.0 - (front - 3.0) * 0.18).max(0.12)
                };
                let name = if (col + row) % 7 == 0 { "teal" } else { "lime" };
                dithered_rect(name, coverage, col as f32 * 120.0, row as f32 * 120.0, 118.0, 118.0)
            })
            .collect();
        let alpha = (t / 0.2).min((SCENES[0] - t) / 0.3).clamp(0.0, 1.0);
        let title = ramp(&frame, 1.1);
        let sub = ramp(&frame, 1.6);
        fframes::svgr!(<g opacity={alpha}>
            {tiles}
            <rect x="140" y="330" width="1640" height="400" rx="24" fill={BG} opacity={0.88 * title} />
            <g opacity={title} transform={Transform::translate(0.0, rise(&frame, 1.1))}>
                <text x="960" y="520" text-anchor="middle" font-family={SANS} font-weight={W_SANS} font-size="176" letter-spacing="-6" fill={INK}>"tantivy-turbo"</text>
            </g>
            <g opacity={sub} transform={Transform::translate(0.0, rise(&frame, 1.6))}>
                <text x="960" y="630" text-anchor="middle" font-family={SANS} font-weight={W_SANS} font-size="52" fill={MUTED}>"A faster Tantivy. Same API. Same results."</text>
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
        // Speedup over Tantivy 0.27, TOP-10, summed time, mean of both run orders.
        let rows = [
            ("Wikipedia 20, total time", 5.2_f32, "5.2x"),
            ("OR, 2 terms", 2.17, "2.2x"),
            ("OR, 3+ terms", 2.01, "2.0x"),
            ("All 1,201 queries", 1.36, "1.36x"),
            ("Phrase", 1.28, "1.3x"),
            ("AND", 0.71, "1.4x slower"),
        ];
        let bars: Vec<Svgr> = rows
            .iter()
            .enumerate()
            .map(|(i, &(name, speedup, label))| {
                let start = 0.6 + i as f32 * 0.12;
                let p = grow(&frame, start);
                let y = 380.0 + i as f32 * 92.0;
                let width = (speedup / 5.4 * 980.0 * p).max(0.5);
                let (fill, coverage) = if speedup < 1.0 { ("slate", 0.6) } else if i == 0 { ("lime", 1.0) } else { ("lime", 0.75) };
                // The fill dissolves in while the bar grows.
                let coverage = coverage * ramp(&frame, start).max(0.15);
                let tone = if speedup < 1.0 { MUTED } else { INK };
                fframes::svgr!(<g opacity={ramp(&frame, start)}>
                    <text x="192" y={y + 40.0} font-family={SANS} font-weight={W_SANS} font-size="40" fill={tone}>{name}</text>
                    {dithered_rect(fill, coverage, 680.0, y, width, 56.0)}
                    <text x={696.0 + width} y={y + 42.0} font-family={MONO} font-weight={W_MONO} font-size="40" fill={tone}>{label}</text>
                </g>)
            })
            .collect();
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[1])}>
            {heading("vs TANTIVY 0.27", "2x faster OR queries", &frame)}
            {bars}
            <text x="192" y="940" font-family={MONO} font-weight={W_MONO} font-size="28" fill={MUTED} opacity={ramp(&frame, 1.8)}>"top-10, summed time; same 1M docs and machine; identical match counts"</text>
        </g>)
    }
}

#[derive(Debug)]
struct VsLucene;

impl Scene for VsLucene {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[2])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let stats = [("2.0x", "faster TOP-10", LIME), ("1.3x", "faster COUNT", TEAL), ("20 / 20", "queries faster", INK)];
        let cards: Vec<Svgr> = stats
            .iter()
            .enumerate()
            .map(|(i, &(value, what, color))| {
                let start = 0.5 + i as f32 * 0.15;
                let x = 192.0 + i as f32 * 524.0;
                let fill = if i == 2 { "ink" } else if i == 1 { "teal" } else { "lime" };
                fframes::svgr!(<g opacity={ramp(&frame, start)} transform={Transform::translate(0.0, rise(&frame, start))}>
                    {dithered_rect(fill, 0.1 * ramp(&frame, start + 0.2), x, 380.0, 488.0, 330.0)}
                    <rect x={x} y="380" width="488" height="330" fill="none" stroke={color} stroke-width="2" />
                    <text x={x + 40.0} y="555" font-family={SANS} font-weight={W_SANS} font-size="120" letter-spacing="-4" fill={color}>{value}</text>
                    <text x={x + 44.0} y="650" font-family={SANS} font-weight={W_SANS} font-size="44" fill={INK}>{what}</text>
                </g>)
            })
            .collect();
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[2])}>
            {heading("vs APACHE LUCENE 10.4", "Beats Lucene on its own scoring", &frame)}
            {cards}
            <text x="192" y="800" font-family={SANS} font-weight={W_SANS} font-size="40" fill={INK} opacity={ramp(&frame, 1.4)}>"~15 MiB resident vs ~617 MiB for the default JVM"</text>
            <text x="192" y="940" font-family={MONO} font-weight={W_MONO} font-size="28" fill={MUTED} opacity={ramp(&frame, 1.6)}>"matched BM25 (k1 1.2, b 0.75), Wikipedia 1M, both process orders"</text>
        </g>)
    }
}

#[derive(Debug)]
struct Trick;

impl Scene for Trick {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(SCENES[3])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        // Regions on one 21-term query: many thin slivers merge into a few wide blocks.
        let merge = grow(&frame, 1.6);
        let count = 30242.0 + (362.0 - 30242.0) * merge;
        let thin = 96;
        let slivers: Vec<Svgr> = (0..thin)
            .map(|i| {
                let w = 1536.0 / thin as f32;
                let coverage = (0.35 + 0.5 * ((i * 7 % 5) as f32 / 4.0)) * (1.0 - merge);
                dithered_rect("slate", coverage * ramp(&frame, 0.5), 192.0 + i as f32 * w, 520.0, w - 3.0, 150.0)
            })
            .collect();
        let wide = [0.0, 0.18, 0.31, 0.52, 0.66, 0.87, 1.0];
        let blocks: Vec<Svgr> = wide
            .windows(2)
            .enumerate()
            .map(|(i, edge)| {
                let x = 192.0 + edge[0] * 1536.0;
                let w = (edge[1] - edge[0]) * 1536.0 - 6.0;
                let name = if i % 2 == 0 { "lime" } else { "teal" };
                dithered_rect(name, 0.8 * merge, x, 520.0, w, 150.0)
            })
            .collect();
        let digits = format_thousands(count.round() as u64);
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[3])}>
            {heading("THE TRICK", "Fewer, wider MaxScore regions", &frame)}
            {slivers}
            {blocks}
            <text x="430" y="460" text-anchor="end" font-family={MONO} font-weight={W_MONO} font-size="64" fill={LIME} opacity={ramp(&frame, 0.5)}>{digits}</text>
            <text x="460" y="460" font-family={SANS} font-weight={W_SANS} font-size="40" fill={INK} opacity={ramp(&frame, 0.5)}>"regions for one 21-term query"</text>
            <text x="192" y="770" font-family={SANS} font-weight={W_SANS} font-size="44" fill={INK} opacity={ramp(&frame, 2.6)}>"Coarse bounds from skip data the index already stores."</text>
            <text x="192" y="840" font-family={SANS} font-weight={W_SANS} font-size="44" fill={INK} opacity={ramp(&frame, 2.9)}>"Switched on only when bookkeeping dominates. 2x faster."</text>
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
        Duration::Seconds(SCENES[4])
    }

    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let rows = [
            ("Bit-identical top-10 and scores", "3 x 1,227 queries"),
            ("Index format unchanged", "0 bytes"),
            ("Peak memory", "+0.23%"),
        ];
        let items: Vec<Svgr> = rows
            .iter()
            .enumerate()
            .map(|(i, &(what, value))| {
                let start = 0.5 + i as f32 * 0.25;
                let y = 420.0 + i as f32 * 150.0;
                let draw = ramp(&frame, start + 0.15);
                fframes::svgr!(<g opacity={ramp(&frame, start)} transform={Transform::translate(0.0, rise(&frame, start))}>
                    {dithered_rect("lime", 0.3, 192.0, y - 60.0, 88.0, 88.0)}
                    <path d={format!("M{} {} l20 20 l40 -44", 214.0, y - 12.0)} stroke={LIME} stroke-width="8" fill="none"
                        stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="90" stroke-dashoffset={90.0 * (1.0 - draw)} />
                    <text x="320" y={y} font-family={SANS} font-weight={W_SANS} font-size="56" fill={INK}>{what}</text>
                    <text x="1728" y={y} text-anchor="end" font-family={MONO} font-weight={W_MONO} font-size="44" fill={LIME}>{value}</text>
                </g>)
            })
            .collect();
        fframes::svgr!(<g opacity={scene_alpha(&frame, SCENES[4])}>
            {heading("NO TRADE-OFFS HIDDEN", "Faster, not different", &frame)}
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
        // Faint, steady dither texture behind the code card (stepping levels full-screen flickers).
        let shimmer = 0.12;
        fframes::svgr!(<g opacity={(t / XFADE).clamp(0.0, 1.0)}>
            {dithered_rect("lime", shimmer, 0.0, 0.0, 1920.0, 940.0)}
            <rect x="192" y="250" width="1536" height="420" rx="20" fill={BG} opacity={card} />
            <rect x="192" y="250" width="1536" height="420" rx="20" fill="none" stroke={SLATE} stroke-width="2" opacity={card} />
            <g opacity={card} transform={Transform::translate(0.0, rise(&frame, 0.3))} font-family={MONO} font-weight={W_MONO} font-size="44">
                <text x="250" y="350" fill={MUTED}>"[dependencies]"</text>
                <text x="250" y="430" fill={INK}>"tantivy = { package = "<tspan fill={LIME}>"\"tantivy-turbo\""</tspan>","</text>
                <text x="250" y="510" fill={INK}>"            git = "<tspan fill={TEAL}>"\"https://github.com/"</tspan></text>
                <text x="250" y="590" fill={TEAL}>"            Priyansh4444/tantivy-turbo\""<tspan fill={INK}>" }"</tspan></text>
            </g>
            <g opacity={ramp(&frame, 1.0)} transform={Transform::translate(0.0, rise(&frame, 1.0))}>
                <text x="960" y="810" text-anchor="middle" font-family={SANS} font-weight={W_SANS} font-size="64" fill={INK}>"Drop-in. Your code keeps using tantivy::"</text>
            </g>
        </g>)
    }
}
