//! Short search story and measured comparison; the document animation is illustrative.
use std::sync::LazyLock;

use fframes::animation::{AnimationRuntime, Easing};
use fframes::{
    AnimateRuntimeInput, AudioMap, Color, Duration, FFramesContext, Frame, Overlap, Scene, Scenes,
    Svgr, Transform, Video, include_media_dir,
};

include_media_dir!(pub struct TantivyTurboDemoMedia, "media");
pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;
const BG: &str = "#000000";
const PANEL: &str = "#10151e";
const LINE: &str = "#374358";
const WHITE: &str = "#f5f9ff";
const DIM: &str = "#a3b4c8";
const BLUE: &str = "#378dff";
const CYAN: &str = "#32f2e2";
const SANS: &str = "DM Sans";
const MONO: &str = "JetBrains Mono";
const XFADE: f32 = 0.2;
const DURATIONS: [f32; 5] = [4.2, 5.2, 5.8, 3.5, 3.0];
static EASE: LazyLock<AnimationRuntime> =
    LazyLock::new(|| AnimationRuntime::new(0.8, &Easing::CubicBezier(0.16, 1.0, 0.3, 1.0)));

pub struct TantivyTurboDemoVideo<'a> {
    pub media: &'a TantivyTurboDemoMedia,
    intro: Intro,
    search: Search,
    comparison: Comparison,
    checked: Checked,
    outro: Outro,
}
impl<'a> TantivyTurboDemoVideo<'a> {
    pub fn new(media: &'a TantivyTurboDemoMedia) -> Self {
        Self {
            media,
            intro: Intro,
            search: Search,
            comparison: Comparison,
            checked: Checked,
            outro: Outro,
        }
    }
}
impl std::fmt::Debug for TantivyTurboDemoVideo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TantivyTurboDemoVideo").finish()
    }
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
            &self.search,
            &self.comparison,
            &self.checked,
            &self.outro,
        ])
    }
    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width={WIDTH} height={HEIGHT} viewBox="0 0 1920 1080">
            <rect width="1920" height="1080" fill={BG} />
            {ctx.render_scenes(&frame)}
        </svg>)
    }
}
fn ramp(frame: &Frame, start: f32) -> f32 {
    let value: f32 = frame.animate_runtime(AnimateRuntimeInput {
        on_second: start,
        from: 0.0,
        to: 1.0,
        animation_runtime: &EASE,
    });
    value.clamp(0.0, 1.0)
}
fn alpha(frame: &Frame, length: f32) -> f32 {
    let t = frame.seconds();
    ((t - XFADE / 2.0) / (XFADE / 2.0))
        .min((length + XFADE / 2.0 - t) / (XFADE / 2.0))
        .clamp(0.0, 1.0)
}
fn line_progress(t: f32, start: f32, duration: f32) -> f32 {
    ((t - start) / duration).clamp(0.0, 1.0)
}

#[derive(Debug)]
struct Intro;
impl Scene for Intro {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(DURATIONS[0])
    }
    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let scan = line_progress(t, 0.8, 1.8);
        let reveal = ramp(&frame, 2.45);
        let docs: Vec<Svgr> = (0..15).map(|i| {
        let col = i % 5; let row = i / 5;
        let x = 770.0 + col as f32 * 176.0; let y = 310.0 + row as f32 * 170.0;
        let hit = i == 2 || i == 8 || i == 10;
        let found = if hit { ramp(&frame, 0.95 + col as f32 * 0.32) } else { 0.0 };
        let scale = ramp(&frame, 0.1 + (i % 5) as f32 * 0.07);
        fframes::svgr!(<g opacity={scale}>
            <rect x={x - 7.0} y={y - 7.0} width="146" height="152" rx="18" fill={BLUE} opacity={found * 0.16} />
            <rect x={x} y={y} width="132" height="138" rx="14" fill={PANEL} stroke={LINE} stroke-width="2" />
            <rect x={x} y={y} width="132" height="138" rx="14" fill="none" stroke={BLUE} stroke-width="3" opacity={found} />
            <line x1={x + 22.0} y1={y + 40.0} x2={x + 94.0} y2={y + 40.0} stroke={if hit { CYAN } else { DIM }} stroke-width="6" stroke-linecap="round" opacity={0.3 + 0.7 * found} />
            <line x1={x + 22.0} y1={y + 66.0} x2={x + 103.0} y2={y + 66.0} stroke={DIM} stroke-width="5" opacity="0.35" stroke-linecap="round" />
            <line x1={x + 22.0} y1={y + 91.0} x2={x + 80.0} y2={y + 91.0} stroke={DIM} stroke-width="5" opacity="0.35" stroke-linecap="round" />
            <circle cx={x + 104.0} cy={y + 113.0} r="7" fill={CYAN} opacity={found} />
        </g>)
    }).collect();
        let bx = 735.0 + scan * 935.0;
        fframes::svgr!(<g opacity={alpha(&frame, DURATIONS[0])}>
        <circle cx="490" cy="530" r="285" fill={BLUE} opacity="0.045" />
        <circle cx="490" cy="530" r="225" fill={BLUE} opacity="0.045" />
        <text x="192" y="270" font-family={MONO} font-size="30" fill={CYAN} opacity={ramp(&frame, 0.1)}>"TANTIVY-TURBO"</text>
        <text x="192" y="385" font-family={SANS} font-weight="600" font-size="86" fill={WHITE} opacity={ramp(&frame, 0.2)}>"Search, faster."</text>
        <rect x="192" y="455" width="458" height="104" rx="24" fill={PANEL} stroke={BLUE} stroke-width="3" />
        <circle cx="245" cy="504" r="15" fill="none" stroke={CYAN} stroke-width="6" />
        <line x1="256" y1="515" x2="269" y2="528" stroke={CYAN} stroke-width="6" stroke-linecap="round" />
        <text x="298" y="524" font-family={SANS} font-size="47" fill={WHITE}>"quokka"</text>
        <line x1="670" y1="507" x2={bx} y2="507" stroke={BLUE} stroke-width="5" opacity="0.75" />
        <circle cx={bx} cy="507" r="23" fill={BLUE} opacity="0.22" />
        <circle cx={bx} cy="507" r="8" fill={CYAN} />
        {docs}
        <line x1={bx} y1="286" x2={bx} y2="822" stroke={CYAN} stroke-width="3" opacity={(1.0 - reveal) * 0.8} />
        <text x="192" y="765" font-family={SANS} font-size="42" fill={DIM} opacity={reveal}>"Find the best matches."</text>
        <text x="1710" y="916" text-anchor="end" font-family={MONO} font-size="26" fill={DIM}>"illustration"</text>
    </g>)
    }
}

#[derive(Debug)]
struct Search;
impl Scene for Search {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(DURATIONS[1])
    }
    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }
    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let find = ramp(&frame, 1.25);
        let keep = ramp(&frame, 2.8);
        let rows = [
            ("quokka", "animal", true, true),
            ("—", "animal", false, false),
            ("quokka", "—", true, false),
            ("quokka", "animal", true, true),
        ];
        let docs: Vec<Svgr> = rows.iter().enumerate().map(|(i, &(a,b,rare,both))| {
        let x = 220.0 + i as f32 * 375.0; let rise = if both { 34.0 * keep } else { -38.0 * keep };
        let opacity = if both { 1.0 } else if rare { 1.0 - 0.72 * keep } else { 1.0 - 0.55 * find };
        let dx = if both { if i == 0 { 55.0 * keep } else { -55.0 * keep } } else { 0.0 };
        let glow = if both { keep } else { 0.0 };
        let enter = ramp(&frame, 0.2 + i as f32 * 0.12);
        fframes::svgr!(<g opacity={opacity * enter} transform={Transform::translate(dx, -rise * 2.0)}>
            <rect x={x - 9.0} y="340" width="318" height="358" rx="28" fill={BLUE} opacity={glow * 0.2} />
            <rect x={x} y="349" width="300" height="340" rx="22" fill={PANEL} stroke={LINE} stroke-width="2" />
            <rect x={x} y="349" width="300" height="340" rx="22" fill="none" stroke={BLUE} stroke-width="4" opacity={if both { keep } else { 0.0 }} />
            <rect x={x + 25.0} y="380" width="42" height="50" rx="5" fill="none" stroke={DIM} stroke-width="3" />
            <line x1={x + 33.0} y1="396" x2={x + 58.0} y2="396" stroke={DIM} stroke-width="3" />
            <line x1={x + 33.0} y1="409" x2={x + 58.0} y2="409" stroke={DIM} stroke-width="3" />
            <text x={x + 28.0} y="510" font-family={SANS} font-size="46" fill={if rare { WHITE } else { DIM }}>{a}</text>
            <text x={x + 28.0} y="578" font-family={SANS} font-size="46" fill={WHITE}>{b}</text>
            <circle cx={x + 255.0} cy="641" r="23" fill={CYAN} opacity={glow} />
            <path d={format!("M{} 641 l10 10 l19 -24",x + 242.0)} stroke={BG} stroke-width="5" fill="none" stroke-linecap="round" stroke-linejoin="round" opacity={glow} />
        </g>)
    }).collect();
        let px = 220.0 + line_progress(t, 0.7, 1.5) * 1500.0;
        fframes::svgr!(<g opacity={alpha(&frame, DURATIONS[1])}>
        <text x="192" y="190" font-family={SANS} font-weight="600" font-size="76" fill={WHITE}>"Find both words."</text>
        <text x="192" y="274" font-family={SANS} font-size="44" fill={DIM}>"quokka "<tspan fill={BLUE}>"+"</tspan>" animal"</text>
        <line x1="220" y1="755" x2={px} y2="755" stroke={BLUE} stroke-width="4" opacity={1.0 - keep} />
        <circle cx={px} cy="755" r="11" fill={CYAN} opacity={1.0 - keep} />
        {docs}
        <text x="192" y="885" font-family={SANS} font-size="46" fill={CYAN} opacity={keep}>"Matches rise to the top."</text>
        <text x="1720" y="930" text-anchor="end" font-family={MONO} font-size="26" fill={DIM}>"illustration"</text>
    </g>)
    }
}

#[derive(Debug)]
struct Comparison;
impl Scene for Comparison {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(DURATIONS[2])
    }
    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }
    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        // Oct 28, port-style, mean of the two run orders' mean-query medians.
        // 168.5 µs versus 136.2 µs. Proportional tracks finish at the same speed.
        let t = frame.seconds();
        let p = line_progress(t, 0.8, 2.0);
        let base = 930.0 * p;
        let fast = (930.0 * p).min(930.0 * 136.2 / 168.5);
        let reveal = ramp(&frame, 2.42);
        let pulse = line_progress(t, 2.42, 0.55);
        let pulse_alpha = line_progress(t, 2.42, 0.08) * (1.0 - pulse);
        fframes::svgr!(<g opacity={alpha(&frame, DURATIONS[2])}>
        <text x="192" y="190" font-family={SANS} font-weight="600" font-size="76" fill={WHITE}>"Top 10 search results."</text>
        <text x="192" y="260" font-family={SANS} font-size="38" fill={CYAN}>"Less time is better"</text>
        <text x="192" y="355" font-family={SANS} font-size="42" fill={DIM}>"lucene-rs"</text>
        <text x="192" y="570" font-family={SANS} font-size="42" fill={WHITE}>"tantivy-turbo"</text>
        <rect x="192" y="390" width="930" height="74" rx="37" fill={PANEL} />
        <rect x="192" y="605" width="930" height="74" rx="37" fill={PANEL} />
        <rect x="192" y="390" width={base.max(1.0)} height="74" rx="37" fill={DIM} />
        <rect x="192" y="605" width={fast.max(1.0)} height="74" rx="37" fill={BLUE} />
        <circle cx={192.0 + fast} cy="642" r="49" fill={BLUE} opacity="0.18" />
        <circle cx={192.0 + fast} cy="642" r={45.0 + pulse * 80.0} fill="none" stroke={CYAN} stroke-width="5" opacity={pulse_alpha} />
        <circle cx={192.0 + fast} cy="642" r="9" fill={CYAN} />
        <text x="1160" y="443" font-family={MONO} font-size="39" fill={DIM} opacity={reveal}>"168.5 µs"</text>
        <text x="1160" y="658" font-family={MONO} font-size="39" fill={WHITE} opacity={reveal}>"136.2 µs"</text>
        <g opacity={reveal} transform={Transform::translate(0.0, (1.0 - reveal) * 35.0)}>
            <circle cx="1534" cy="523" r="162" fill={BLUE} opacity="0.10" />
            <circle cx="1534" cy="523" r="140" fill={PANEL} stroke={BLUE} stroke-width="4" />
            <text x="1534" y="524" text-anchor="middle" font-family={SANS} font-weight="600" font-size="83" fill={WHITE}>"1.24×"</text>
            <text x="1534" y="577" text-anchor="middle" font-family={SANS} font-size="40" fill={CYAN}>"faster"</text>
        </g>
        <text x="192" y="860" font-family={MONO} font-size="34" fill={DIM}>"1,201 searches · warm · 1 core · Oct 28"</text>
        <text x="192" y="910" font-family={MONO} font-size="27" fill={DIM}>"mean query medians · animated scale"</text>
    </g>)
    }
}

#[derive(Debug)]
struct Checked;
impl Scene for Checked {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(DURATIONS[3])
    }
    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }
    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let join = ramp(&frame, 0.45);
        let check = ramp(&frame, 1.25);
        let dx = 240.0 * (1.0 - join);
        fframes::svgr!(<g opacity={alpha(&frame, DURATIONS[3])}>
        <circle cx="960" cy="430" r="245" fill={BLUE} opacity="0.06" />
        <rect x={730.0 - dx} y="325" width="180" height="200" rx="22" fill={PANEL} stroke={BLUE} stroke-width="4" opacity={1.0 - check} />
        <rect x={1010.0 + dx} y="325" width="180" height="200" rx="22" fill={PANEL} stroke={CYAN} stroke-width="4" opacity={1.0 - check} />
        <line x1={760.0 - dx} y1="385" x2={880.0 - dx} y2="385" stroke={DIM} stroke-width="6" opacity={1.0 - check} />
        <line x1={1040.0 + dx} y1="385" x2={1160.0 + dx} y2="385" stroke={DIM} stroke-width="6" opacity={1.0 - check} />
        <circle cx="960" cy="430" r={(120.0 * check).max(0.1)} fill={BLUE} opacity="0.16" />
        <circle cx="960" cy="430" r={(96.0 * check).max(0.1)} fill={PANEL} stroke={CYAN} stroke-width="5" opacity={check} />
        <path d="M 910 428 l 35 35 l 70 -80" fill="none" stroke={CYAN} stroke-width="14" stroke-linecap="round" stroke-linejoin="round" opacity={check} />
        <text x="960" y="690" text-anchor="middle" font-family={SANS} font-weight="600" font-size="72" fill={WHITE} opacity={check}>"14,400 checks"</text>
        <text x="960" y="762" text-anchor="middle" font-family={SANS} font-size="40" fill={DIM} opacity={check}>"against a full scan"</text>
    </g>)
    }
}

#[derive(Debug)]
struct Outro;
impl Scene for Outro {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(DURATIONS[4])
    }
    fn overlap(&self) -> Overlap {
        Overlap::Previous(XFADE)
    }
    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let p = ramp(&frame, 0.2);
        let t = frame.seconds();
        let ring = 160.0 + 70.0 * line_progress(t, 0.4, 1.5);
        fframes::svgr!(<g opacity={alpha(&frame, DURATIONS[4])}>
        <circle cx="960" cy="440" r={ring} fill="none" stroke={BLUE} stroke-width="3" opacity={0.35 * (1.0 - line_progress(t, 0.4, 1.5))} />
        <circle cx="960" cy="440" r="170" fill={BLUE} opacity="0.06" />
        <text x="960" y="450" text-anchor="middle" font-family={SANS} font-weight="600" font-size="88" fill={WHITE} opacity={p}>"tantivy"<tspan fill={BLUE}>"-turbo"</tspan></text>
        <text x="960" y="530" text-anchor="middle" font-family={SANS} font-size="43" fill={DIM} opacity={p}>"Fast search in Rust."</text>
        <text x="960" y="815" text-anchor="middle" font-family={MONO} font-size="40" fill={CYAN} opacity={p}>"github.com/Priyansh4444/tantivy-turbo"</text>
    </g>)
    }
}
