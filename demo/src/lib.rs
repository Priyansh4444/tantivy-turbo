//! Measured Lucene comparison. Values and limitations are documented in the benchmark reports.
use fframes::{AudioMap, Color, Duration, FFramesContext, Frame, Svgr, Video, include_media_dir};
include_media_dir!(pub struct TantivyTurboDemoMedia, "media");
pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;
const INK: &str = "#f5f8ff";
const DIM: &str = "#aebed3";
const BLUE: &str = "#568cff";
const AQUA: &str = "#4cf4df";
const PANEL: &str = "#0c1422";
const BORDER: &str = "#314762";
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
fn smooth(t: f32, a: f32, b: f32) -> f32 {
    let x = ((t - a) / (b - a)).clamp(0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}
fn window(t: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    smooth(t, a, b) * (1.0 - smooth(t, c, d))
}
impl Video for TantivyTurboDemoVideo<'_> {
    const FPS: usize = 60;
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const BACKGROUND_COLOR: Color = Color::BLACK;
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(23.0)
    }
    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }
    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let java = window(t, 0.0, 0.55, 5.0, 5.65);
        let memory = window(t, 5.65, 6.25, 10.3, 10.95);
        let proof = window(t, 10.95, 11.55, 13.8, 14.45);
        let port = window(t, 14.45, 15.05, 19.1, 19.75);
        let finish = smooth(t, 19.75, 20.4);
        let j = smooth(t, 0.7, 2.3);
        let m = smooth(t, 6.3, 7.9);
        let p1 = smooth(t, 15.15, 16.85);
        let p2 = smooth(t, 15.15, 16.53);
        let check = smooth(t, 11.5, 12.5);
        let stars:Vec<Svgr>=(0..64).map(|i| {
   let x=110.0+((i*397)%1690) as f32; let y=90.0+((i*193)%860) as f32;
   let a=0.10+0.06*(t*0.22+i as f32*1.37).sin();
   fframes::svgr!(<circle cx={x} cy={y} r={if i%9==0 {2.0} else {1.2}} fill={AQUA} opacity={a}/>)
  }).collect();
        let glow:Vec<Svgr>=(0..8).map(|i| {
   let r=470.0-i as f32*48.0;
   fframes::svgr!(<circle cx={1260.0+35.0*(t*0.14).sin()} cy="500" r={r} fill={BLUE} opacity="0.012"/>)
  }).collect();
        let jw = (700.0 * j).max(0.1);
        let bw = (700.0 * 0.478 * j).max(0.1);
        let band = (700.0 * 0.071 * j).max(0.1);
        let mw = (700.0 * m).max(0.1);
        let tw = (700.0 * (14.9 / 346.27) * m).max(0.1);
        let pw1 = (700.0 * p1).max(0.1);
        let pw2 = (700.0 * (136.2 / 168.5) * p2).max(0.1);
        fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width={WIDTH} height={HEIGHT} viewBox="0 0 1920 1080">
   <rect width="1920" height="1080" fill="#000000" />{glow}{stars}
   <path d="M140 948 H1780" stroke={BORDER} stroke-width="1" opacity="0.48"/>
   <g opacity={java}>
    <text x="160" y="150" font-family={MONO} font-size="27" fill={AQUA}>"APACHE LUCENE 10.4 / JAVA"</text>
    <text x="160" y="255" font-family={SANS} font-weight="600" font-size="70" fill={INK}>"Search, faster."</text>
    <text x="160" y="315" font-family={SANS} font-size="32" fill={DIM}>"Top 10 results · less time is better"</text>
    <text x="160" y="390" font-family={SANS} font-size="36" fill={DIM}>"Lucene"</text>
    <rect x="160" y="412" width="700" height="62" rx="31" fill={PANEL}/>
    <rect x="160" y="412" width={jw} height="62" rx="31" fill={DIM}/>
    <text x="160" y="560" font-family={SANS} font-size="36" fill={INK}>"tantivy-turbo"</text>
    <rect x="160" y="582" width="700" height="62" rx="31" fill={PANEL}/>
    <rect x="160" y="582" width={bw} height="62" rx="31" fill={BLUE}/>
    <rect x={160.0+bw} y="582" width={band} height="62" fill={AQUA} opacity="0.55"/>
    <g opacity={smooth(t,1.6,2.5)}>
     <text x="1070" y="460" font-family={SANS} font-weight="600" font-size="86" fill={INK}>"1.82–2.09×"</text>
     <text x="1075" y="525" font-family={SANS} font-size="43" fill={AQUA}>"faster"</text>
    </g>
    <text x="160" y="824" font-family={MONO} font-size="29" fill={DIM}>"20 searches · older build · Oct 4"</text>
    <text x="160" y="869" font-family={MONO} font-size="25" fill={DIM}>"geometric mean · two process orders"</text>
   </g>
   <g opacity={memory}>
    <text x="160" y="150" font-family={MONO} font-size="27" fill={AQUA}>"APACHE LUCENE / MEMORY"</text>
    <text x="160" y="255" font-family={SANS} font-weight="600" font-size="70" fill={INK}>"A smaller footprint."</text>
    <text x="160" y="315" font-family={SANS} font-size="32" fill={DIM}>"Warmed search process · resident memory"</text>
    <text x="160" y="390" font-family={SANS} font-size="36" fill={DIM}>"Lucene"</text>
    <rect x="160" y="412" width="700" height="62" rx="31" fill={PANEL}/>
    <rect x="160" y="412" width={mw} height="62" rx="31" fill={DIM}/>
    <text x="895" y="459" font-family={MONO} font-size="36" fill={DIM} opacity={smooth(t,7.0,7.8)}>"346 MiB"</text>
    <text x="160" y="560" font-family={SANS} font-size="36" fill={INK}>"tantivy-turbo"</text>
    <rect x="160" y="582" width="700" height="62" rx="31" fill={PANEL}/>
    <rect x="160" y="582" width={tw} height="62" rx="31" fill={BLUE}/>
    <circle cx={160.0+tw} cy="613" r="7" fill={AQUA}/>
    <text x="895" y="629" font-family={MONO} font-size="36" fill={INK} opacity={smooth(t,7.0,7.8)}>"15 MiB"</text>
    <text x="160" y="824" font-family={MONO} font-size="29" fill={DIM}>"older build · Oct 4 · default JVM heap"</text>
    <text x="160" y="869" font-family={MONO} font-size="25" fill={DIM}>"includes mapped pages · not a controlled-heap test"</text>
   </g>
   <g opacity={proof}>
    <circle cx="960" cy="400" r="202" fill={BLUE} opacity="0.08"/>
    <circle cx="960" cy="400" r="160" fill={PANEL} stroke={BLUE} stroke-width="3"/>
    <path d="M 900 401 l 42 42 l 79 -93" fill="none" stroke={AQUA} stroke-width="14" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="215" stroke-dashoffset={215.0*(1.0-check)}/>
    <text x="960" y="690" text-anchor="middle" font-family={SANS} font-weight="600" font-size="76" fill={INK}>"14,400 checks"</text>
    <text x="960" y="765" text-anchor="middle" font-family={SANS} font-size="40" fill={DIM}>"against a full scan"</text>
   </g>
   <g opacity={port}>
    <text x="160" y="150" font-family={MONO} font-size="27" fill={AQUA}>"ALSO / LUCENE-RS RUST PORT"</text>
    <text x="160" y="255" font-family={SANS} font-weight="600" font-size="70" fill={INK}>"Against lucene-rs."</text>
    <text x="160" y="315" font-family={SANS} font-size="32" fill={DIM}>"Top 10 results · less time is better"</text>
    <text x="160" y="390" font-family={SANS} font-size="36" fill={DIM}>"lucene-rs"</text>
    <rect x="160" y="412" width="700" height="62" rx="31" fill={PANEL}/>
    <rect x="160" y="412" width={pw1} height="62" rx="31" fill={DIM}/>
    <text x="160" y="560" font-family={SANS} font-size="36" fill={INK}>"tantivy-turbo"</text>
    <rect x="160" y="582" width="700" height="62" rx="31" fill={PANEL}/>
    <rect x="160" y="582" width={pw2} height="62" rx="31" fill={BLUE}/>
    <g opacity={smooth(t,16.3,17.1)}>
     <text x="895" y="459" font-family={MONO} font-size="34" fill={DIM}>"168.5 µs"</text>
     <text x="895" y="629" font-family={MONO} font-size="34" fill={INK}>"136.2 µs"</text>
     <text x="1250" y="470" font-family={SANS} font-weight="600" font-size="90" fill={INK}>"1.24×"</text>
     <text x="1255" y="535" font-family={SANS} font-size="42" fill={AQUA}>"faster"</text>
    </g>
    <text x="160" y="824" font-family={MONO} font-size="29" fill={DIM}>"1,201 searches · warm · 1 core · Oct 28"</text>
    <text x="160" y="869" font-family={MONO} font-size="25" fill={DIM}>"mean query medians · animated scale"</text>
   </g>
   <g opacity={finish}>
    <circle cx="960" cy="415" r="235" fill={BLUE} opacity="0.07"/>
    <text x="960" y="430" text-anchor="middle" font-family={SANS} font-weight="600" font-size="92" fill={INK}>"tantivy"<tspan fill={BLUE}>"-turbo"</tspan></text>
    <text x="960" y="515" text-anchor="middle" font-family={SANS} font-size="48" fill={INK}>"Faster search. Same Tantivy API."</text>
    <text x="960" y="815" text-anchor="middle" font-family={MONO} font-size="35" fill={AQUA}>"github.com/Priyansh4444/tantivy-turbo"</text>
   </g>
  </svg>)
    }
}
