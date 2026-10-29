//! Motion-led search story. Search visuals are illustrative; measurements are retained data.
use fframes::{AudioMap, Color, Duration, FFramesContext, Frame, Svgr, Video, include_media_dir};

include_media_dir!(pub struct TantivyTurboDemoMedia, "media");
pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;
const DURATION: f32 = 26.0;
const BLACK: &str = "#000000";
const WHITE: &str = "#f4f9ff";
const MUTED: &str = "#b0c0d4";
const BLUE: &str = "#498dff";
const CYAN: &str = "#43f5e5";
const PANEL: &str = "#101724";
const GRID: &str = "#364a65";
const SANS: &str = "DM Sans";
const MONO: &str = "JetBrains Mono";

pub struct TantivyTurboDemoVideo<'a> {
    pub media: &'a TantivyTurboDemoMedia,
}
impl<'a> TantivyTurboDemoVideo<'a> {
    pub fn new(media: &'a TantivyTurboDemoMedia) -> Self {
        Self { media }
    }
}
impl std::fmt::Debug for TantivyTurboDemoVideo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TantivyTurboDemoVideo").finish()
    }
}

/// Smooth, bounded transitions keep position and velocity continuous at each endpoint.
fn ease(t: f32, start: f32, end: f32) -> f32 {
    let x = ((t - start) / (end - start)).clamp(0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}
fn window(t: f32, enter: (f32, f32), exit: (f32, f32)) -> f32 {
    ease(t, enter.0, enter.1) * (1.0 - ease(t, exit.0, exit.1))
}
fn mix(a: f32, b: f32, p: f32) -> f32 {
    a + (b - a) * p
}

impl Video for TantivyTurboDemoVideo<'_> {
    const FPS: usize = 60;
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const BACKGROUND_COLOR: Color = Color::BLACK;
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(DURATION)
    }
    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }
    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let field = window(t, (0.0, 0.45), (8.35, 9.15));
        let field_copy = window(t, (0.0, 0.45), (7.35, 8.1));
        let opening = window(t, (0.15, 0.8), (3.6, 4.4));
        let focused = window(t, (3.6, 4.4), (7.5, 8.6));
        let bench = window(t, (7.7, 8.9), (12.85, 13.55));
        let bench_copy = ease(t, 8.35, 9.05);
        let lucene = window(t, (12.85, 13.55), (17.85, 18.55));
        let lucene_copy = ease(t, 13.5, 14.15);
        let proof = window(t, (17.85, 18.55), (21.8, 22.7));
        let proof_copy = window(t, (18.65, 19.35), (21.55, 22.15));
        let end = ease(t, 21.8, 22.7);
        let end_copy = ease(t, 22.35, 23.05);
        let gather = ease(t, 3.8, 5.6);
        let scan = ease(t, 1.45, 3.65);
        let origin_x = 750.0 + scan * 870.0;
        let query = "quokka + animal";
        let typed = (query.len() as f32 * ease(t, 0.35, 1.45)) as usize;
        let ambiance: Vec<Svgr> = (0..48).map(|i| {
            let x = 118.0 + ((i * 239) % 1650) as f32;
            let y = 102.0 + ((i * 163) % 850) as f32;
            let shimmer = 0.10 + 0.07 * (t * 0.55 + i as f32 * 1.3).sin();
            fframes::svgr!(<circle cx={x} cy={y} r={if i % 7 == 0 {2.0} else {1.1}} fill={CYAN} opacity={shimmer} />)
        }).collect();
        let glows: Vec<Svgr> = (0..7).map(|i| {
            let radius = 480.0 - i as f32 * 50.0;
            fframes::svgr!(<circle cx={1240.0 + 45.0 * (t * 0.15).sin()} cy="510" r={radius} fill={BLUE} opacity="0.010" />)
        }).collect();
        let routes: Vec<Svgr> = (0..3)
            .map(|i| {
                let x = 860.0;
                let y = 390.0 + i as f32 * 145.0;
                let guide = window(t, (3.9, 4.6), (6.6, 7.6));
                let travel = ease(t, 4.0 + i as f32 * 0.12, 5.55 + i as f32 * 0.12);
                let px = mix(685.0, x, travel);
                let control_y = 280.0 - 96.0 * i as f32;
                let py = 481.0 * (1.0 - travel).powi(2)
                    + 2.0 * control_y * travel * (1.0 - travel)
                    + y * travel * travel;
                let arc = format!("M685 481 Q{} {control_y} {x} {y}", mix(685.0, x, 0.5));
                fframes::svgr!(<g opacity={guide * field}>
                <path d={arc} fill="none" stroke={BLUE} stroke-width="2" opacity="0.46" />
                <circle cx={px} cy={py} r="24" fill={BLUE} opacity="0.16" />
                <circle cx={px} cy={py} r="5" fill={CYAN} />
            </g>)
            })
            .collect();
        // Background cards render first, so they cannot obscure result text.
        let doc_order = (0..24)
            .filter(|i| ![3, 10, 20].contains(i))
            .chain([3, 10, 20]);
        let docs: Vec<Svgr> = doc_order.map(|i| {
            let col = i % 6; let row = i / 6;
            let base_x = 750.0 + col as f32 * 143.0;
            let base_y = 255.0 + row as f32 * 143.0;
            let order = match i { 3 => Some(0), 10 => Some(1), 20 => Some(2), _ => None };
            let is_hit = order.is_some();
            let rank_index = order.unwrap_or(0);
            let rank = rank_index as f32;
            let label = ["Quokka habitat", "Quokka animal facts", "Quokka field notes"][rank_index];
            let selected = if is_hit { ease(t, 1.55 + col as f32 * 0.30, 2.05 + col as f32 * 0.30) } else { 0.0 };
            let arrival = if is_hit { ease(t, 3.7 + rank * 0.12, 5.45 + rank * 0.12) } else { gather };
            let x = if is_hit { mix(base_x, 860.0, arrival) } else { base_x + (base_x - 1130.0) * 0.13 * gather };
            let y = if is_hit { mix(base_y, 330.0 + rank * 145.0, arrival) } else { base_y + 100.0 * gather };
            let w = if is_hit { mix(112.0, 720.0, arrival) } else { 112.0 };
            let h = if is_hit { mix(118.0, 120.0, arrival) } else { 118.0 };
            let opacity = if is_hit { field } else { field * (1.0 - 0.92 * gather) };
            let text_alpha = ease(t, 5.3 + rank * 0.12, 5.8 + rank * 0.12) * selected;
            let line_alpha = if is_hit { 1.0 - ease(t, 4.9 + rank * 0.12, 5.2 + rank * 0.12) } else { 1.0 };
            let sparkle = (1.0 - ease(t, 5.15, 6.2)) * gather * selected;
            fframes::svgr!(<g opacity={opacity}>
                <rect x={x - 7.0} y={y - 7.0} width={w + 14.0} height={h + 14.0} rx="19" fill={BLUE} opacity={selected * 0.18} />
                <rect x={x} y={y} width={w} height={h} rx="14" fill={PANEL} stroke={GRID} stroke-width="2" />
                <rect x={x} y={y} width={w} height={h} rx="14" fill="none" stroke={BLUE} stroke-width="3" opacity={selected} />
                <line x1={x + 20.0} y1={y + 34.0} x2={x + w * 0.68} y2={y + 34.0} stroke={if is_hit { CYAN } else { MUTED }} stroke-width="5" stroke-linecap="round" opacity={(0.34 + selected * 0.66) * line_alpha} />
                <line x1={x + 20.0} y1={y + 56.0} x2={x + w * 0.76} y2={y + 56.0} stroke={MUTED} stroke-width="4" stroke-linecap="round" opacity={0.30 * line_alpha} />
                <line x1={x + 20.0} y1={y + 78.0} x2={x + w * 0.58} y2={y + 78.0} stroke={MUTED} stroke-width="4" stroke-linecap="round" opacity={0.30 * line_alpha} />
                <text x={x + 24.0} y={y + 72.0} font-family={MONO} font-size="29" fill={CYAN} opacity={text_alpha}>{format!("0{}", rank_index + 1)}</text>
                <text x={x + 105.0} y={y + 49.0} font-family={SANS} font-size="31" fill={WHITE} opacity={text_alpha}>{label}</text>
                <text x={x + 105.0} y={y + 86.0} font-family={SANS} font-size="24" fill={MUTED} opacity={text_alpha}>"quokka · animal"</text>
                <circle cx={x + w - 34.0} cy={y + h / 2.0} r="7" fill={CYAN} opacity={selected} />
                <circle cx={x + w - 34.0} cy={y + h / 2.0} r={15.0 + 20.0 * gather} fill="none" stroke={CYAN} stroke-width="3" opacity={sparkle} />
            </g>)
        }).collect();
        let baseline = ease(t, 9.15, 11.55);
        let turbo = ease(t, 9.15, 11.15);
        let base_width = (730.0 * baseline).max(0.1);
        let fast_width = (730.0 * (136.2 / 168.5) * turbo).max(0.1);
        let result = ease(t, 11.0, 11.8);
        let check = ease(t, 19.05, 20.2);
        let converge = ease(t, 18.35, 19.55);
        let streams: Vec<Svgr> = (0..3)
            .map(|i| {
                let y = 323.0 + 52.0 * i as f32;
                let a = mix(490.0, 850.0, converge);
                let b = mix(1320.0, 1010.0, converge);
                let opacity = proof * (1.0 - ease(t, 19.15, 19.75));
                fframes::svgr!(<g opacity={opacity}>
                <rect x={a} y={y} width="110" height="29" rx="14" fill={BLUE} opacity="0.35" />
                <rect x={b} y={y} width="110" height="29" rx="14" fill={CYAN} opacity="0.35" />
            </g>)
            })
            .collect();
        let intro_y = 290.0 - 25.0 * ease(t, 0.1, 0.9);
        fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width={WIDTH} height={HEIGHT} viewBox="0 0 1920 1080">
          <rect width="1920" height="1080" fill={BLACK} />
          {glows}
          <path d="M110 895 H1810 M110 901 H1810" stroke={GRID} stroke-width="1" opacity="0.25" />
          {ambiance}
          <g opacity={field_copy}>
            <text x="160" y="153" font-family={MONO} font-size="26" fill={CYAN}>"TANTIVY-TURBO / SEARCH"</text>
            <text x="160" y={intro_y} font-family={SANS} font-weight="600" font-size="82" fill={WHITE} opacity={opening}>"Search, faster."</text>
            <text x="160" y="265" font-family={SANS} font-weight="600" font-size="82" fill={WHITE} opacity={focused}>"Your top results."</text>
            <rect x="160" y="425" width="510" height="112" rx="22" fill={BLUE} opacity="0.14" />
            <rect x="164" y="429" width="502" height="104" rx="20" fill={PANEL} stroke={BLUE} stroke-width="3" />
            <circle cx="220" cy="476" r="15" fill="none" stroke={CYAN} stroke-width="5" />
            <line x1="231" y1="487" x2="244" y2="500" stroke={CYAN} stroke-width="5" stroke-linecap="round" />
            <text x="272" y="493" font-family={SANS} font-size="42" fill={WHITE}>{&query[..typed]}</text>
            <path d={format!("M680 481 H{origin_x}")} fill="none" stroke={BLUE} stroke-width="4" opacity={(1.0 - gather) * 0.85} />
            <circle cx={origin_x} cy="481" r="25" fill={BLUE} opacity={(1.0 - gather) * 0.22} />
            <circle cx={origin_x} cy="481" r="7" fill={CYAN} opacity={1.0 - gather} />
            <line x1={origin_x} y1="210" x2={origin_x} y2="860" stroke={CYAN} stroke-width="2" opacity={(1.0 - gather) * 0.55} />
            <text x="160" y="850" font-family={MONO} font-size="26" fill={MUTED}>"ILLUSTRATIVE SEARCH"</text>
          </g>
          <g opacity={field * gather}>
            <rect x="824" y="245" width="792" height="580" rx="24" fill={PANEL} stroke={GRID} stroke-width="2" opacity="0.78" />
            <circle cx="866" cy="291" r="5" fill={CYAN} />
            <circle cx="888" cy="291" r="5" fill={BLUE} opacity="0.6" />
            <circle cx="910" cy="291" r="5" fill={MUTED} opacity="0.5" />
          </g>
          {routes}
          {docs}
          <g opacity={bench}>
            <text x="160" y="170" font-family={MONO} font-size="27" fill={CYAN} opacity={bench_copy}>"SEARCH / TOP 10 RESULTS"</text>
            <text x="160" y="278" font-family={SANS} font-weight="600" font-size="82" fill={WHITE} opacity={bench_copy}>"Less waiting."</text>
            <text x="160" y="326" font-family={SANS} font-size="30" fill={CYAN} opacity={bench_copy}>"Less time is better"</text>
            <text x="160" y="386" font-family={SANS} font-size="40" fill={MUTED} opacity={bench_copy}>"lucene-rs"</text>
            <text x="160" y="580" font-family={SANS} font-size="40" fill={WHITE} opacity={bench_copy}>"tantivy-turbo"</text>
            <rect x="160" y="415" width="730" height="62" rx="31" fill={PANEL} />
            <rect x="160" y="609" width="730" height="62" rx="31" fill={PANEL} />
            <rect x="160" y="415" width={base_width} height="62" rx="31" fill={MUTED} />
            <rect x="160" y="609" width={fast_width} height="62" rx="31" fill={BLUE} />
            <circle cx={160.0 + fast_width} cy="640" r="37" fill={BLUE} opacity="0.20" />
            <circle cx={160.0 + fast_width} cy="640" r="7" fill={CYAN} />
            <text x="930" y="461" font-family={MONO} font-size="36" fill={MUTED} opacity={result}>"168.5 µs"</text>
            <text x="930" y="655" font-family={MONO} font-size="36" fill={WHITE} opacity={result}>"136.2 µs"</text>
            <g opacity={result}>
              <circle cx="1480" cy="520" r="181" fill={BLUE} opacity="0.08" />
              <circle cx="1480" cy="520" r="158" fill={PANEL} stroke={BLUE} stroke-width="4" />
              <text x="1480" y="522" text-anchor="middle" font-family={SANS} font-weight="600" font-size="89" fill={WHITE}>"1.24×"</text>
              <text x="1480" y="578" text-anchor="middle" font-family={SANS} font-size="39" fill={CYAN}>"faster"</text>
            </g>
            <text x="160" y="810" font-family={MONO} font-size="31" fill={MUTED} opacity={bench_copy}>"1,201 searches · warm · 1 core · Oct 28"</text>
            <text x="160" y="855" font-family={MONO} font-size="25" fill={MUTED} opacity={bench_copy}>"mean query medians · animated comparison"</text>
          </g>
          <g opacity={lucene}>
            <text x="160" y="170" font-family={MONO} font-size="27" fill={CYAN} opacity={lucene_copy}>"APACHE LUCENE 10.4 / JAVA"</text>
            <text x="160" y="280" font-family={SANS} font-weight="600" font-size="76" fill={WHITE} opacity={lucene_copy}>"The core behind search."</text>
            <text x="160" y="360" font-family={SANS} font-size="42" fill={MUTED} opacity={lucene_copy}>"Elasticsearch · OpenSearch · Solr"</text>
            <circle cx="1390" cy="530" r={mix(120.0, 195.0, lucene_copy)} fill={BLUE} opacity="0.08" />
            <circle cx="1390" cy="530" r={mix(105.0, 168.0, lucene_copy)} fill={PANEL} stroke={BLUE} stroke-width="4" />
            <text x="1390" y="527" text-anchor="middle" font-family={SANS} font-weight="600" font-size="61" fill={WHITE} opacity={lucene_copy}>"1.82–2.09×"</text>
            <text x="1390" y="581" text-anchor="middle" font-family={SANS} font-size="36" fill={CYAN} opacity={lucene_copy}>"faster"</text>
            <text x="160" y="595" font-family={SANS} font-size="48" fill={WHITE} opacity={lucene_copy}>"Typical top-10 search"</text>
            <text x="160" y="810" font-family={MONO} font-size="31" fill={MUTED} opacity={lucene_copy}>"20 queries · older build · Oct 4"</text>
            <text x="160" y="855" font-family={MONO} font-size="25" fill={MUTED} opacity={lucene_copy}>"geometric mean · two process orders"</text>
          </g>
          {streams}
          <g opacity={proof}>
            <circle cx="960" cy="385" r="205" fill={BLUE} opacity="0.09" />
            <circle cx="960" cy="385" r="150" fill={PANEL} stroke={BLUE} stroke-width="3" />
            <circle cx="960" cy="385" r="119" fill="none" stroke={CYAN} stroke-width="3" opacity={check} />
            <path d="M900 385 l42 42 l78 -92" fill="none" stroke={CYAN} stroke-width="15" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="215" stroke-dashoffset={215.0 * (1.0 - check)} />
          </g>
          <g opacity={proof_copy}>
            <text x="960" y="665" text-anchor="middle" font-family={SANS} font-weight="600" font-size="78" fill={WHITE}>"14,400 checks"</text>
            <text x="960" y="740" text-anchor="middle" font-family={SANS} font-size="42" fill={MUTED}>"against a full scan"</text>
          </g>
          <g opacity={end}>
            <circle cx="960" cy="425" r="250" fill={BLUE} opacity="0.07" />
            <text x="960" y="423" text-anchor="middle" font-family={SANS} font-weight="600" font-size="95" fill={WHITE} opacity={end_copy}>"tantivy"<tspan fill={BLUE}>"-turbo"</tspan></text>
            <text x="960" y="520" text-anchor="middle" font-family={SANS} font-size="51" fill={WHITE} opacity={end_copy}>"Faster search. Same Tantivy API."</text>
            <text x="960" y="808" text-anchor="middle" font-family={MONO} font-size="37" fill={CYAN} opacity={end_copy}>"github.com/Priyansh4444/tantivy-turbo"</text>
          </g>
        </svg>)
    }
}
