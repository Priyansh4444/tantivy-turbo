//! Short, continuous search story. Example results are illustrative.
use fframes::{AudioMap, Color, Duration, FFramesContext, Frame, Svgr, Video, include_media_dir};
include_media_dir!(pub struct TantivyTurboDemoMedia, "media");
pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;
const LENGTH: f32 = 21.0;
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
fn lerp(a: f32, b: f32, p: f32) -> f32 {
    a + (b - a) * p
}

impl Video for TantivyTurboDemoVideo<'_> {
    const FPS: usize = 60;
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const BACKGROUND_COLOR: Color = Color::BLACK;
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(LENGTH)
    }
    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }
    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let search = window(t, 0.0, 0.45, 5.55, 6.0);
        let java = window(t, 6.0, 6.55, 10.0, 10.7);
        let port = window(t, 10.65, 11.05, 13.8, 14.5);
        let proof = window(t, 14.05, 14.8, 17.3, 18.0);
        let finish = smooth(t, 17.95, 18.35);
        let input = "quokka animal";
        let n = (input.len() as f32 * smooth(t, 0.45, 1.55)) as usize;
        let trace = smooth(t, 1.25, 4.0);
        let drift = t * 0.22;
        let particles:Vec<Svgr>=(0..64).map(|i| {
   let x=110.0+((i*397)%1690) as f32;
   let y=90.0+((i*193)%860) as f32;
   let a=0.08+0.06*(drift+i as f32*1.37).sin();
   fframes::svgr!(<circle cx={x} cy={y} r={if i%9==0 {2.0} else {1.2}} fill={AQUA} opacity={a}/>)
  }).collect();
        let glow:Vec<Svgr>=(0..8).map(|i| {
   let r=470.0-i as f32*48.0;
   fframes::svgr!(<circle cx={1260.0+35.0*(t*0.14).sin()} cy="500" r={r} fill={BLUE} opacity="0.012"/>)
  }).collect();
        let outputs = [
            ("Quokka habitat", "island wildlife"),
            ("Quokka facts", "animal profile"),
            ("Field notes", "quokka research"),
        ];
        let results:Vec<Svgr>=outputs.iter().enumerate().map(|(i,&(title,subtitle))| {
   let delay=i as f32*0.18;
   let show=smooth(t,2.7+delay,3.6+delay);
   let y=448.0+i as f32*116.0+35.0*(1.0-show);
   fframes::svgr!(<g opacity={show}>
    <rect x="380" y={y} width="1160" height="96" rx="19" fill={PANEL} stroke={BORDER} stroke-width="2" />
    <rect x="380" y={y} width="6" height="96" rx="3" fill={AQUA} />
    <text x="424" y={y+59.0} font-family={MONO} font-size="28" fill={AQUA}>{format!("0{}",i+1)}</text>
    <text x="510" y={y+43.0} font-family={SANS} font-size="34" fill={INK}>{title}</text>
    <text x="510" y={y+76.0} font-family={SANS} font-size="25" fill={DIM}>{subtitle}</text>
    <circle cx="1488" cy={y+48.0} r="8" fill={AQUA}/>
   </g>)
  }).collect();
        let networks:Vec<Svgr>=(0..3).map(|i| {
    let x=260.0+i as f32*260.0;
    let p=smooth(t,6.45+i as f32*0.1,7.65+i as f32*0.1);
    let travel=smooth(t,6.7+i as f32*0.12,8.1+i as f32*0.12);
    let dot_x=520.0*(1.0-travel)*(1.0-travel)+2.0*x*travel*(1.0-travel)+x*travel*travel;
    let dot_y=480.0*(1.0-travel)*(1.0-travel)+840.0*travel*(1.0-travel)+585.0*travel*travel;
    fframes::svgr!(<g opacity={p}>
      <path d={format!("M 520 480 Q {x} 420 {x} 585")} fill="none" stroke={BLUE} stroke-width="2" opacity="0.60" />
      <circle cx={dot_x} cy={dot_y} r="17" fill={BLUE} opacity="0.2" />
      <circle cx={dot_x} cy={dot_y} r="5" fill={AQUA} />
      <circle cx={x} cy="600" r="34" fill={PANEL} stroke={BLUE} stroke-width="2" />
      <circle cx={x} cy="600" r="6" fill={AQUA}/>
    </g>)
  }).collect();
        let gate = smooth(t, 7.2, 8.0);
        let p1 = smooth(t, 11.2, 12.9);
        let p2 = smooth(t, 11.2, 12.58);
        let w1 = (680.0 * p1).max(0.1);
        let w2 = (680.0 * (136.2 / 168.5) * p2).max(0.1);
        let score = smooth(t, 12.25, 12.95);
        let checked = smooth(t, 15.4, 16.45);
        let ring = lerp(75.0, 160.0, smooth(t, 14.4, 15.2));
        fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width={WIDTH} height={HEIGHT} viewBox="0 0 1920 1080">
   <rect width="1920" height="1080" fill="#000000" />
   {glow}{particles}
   <path d="M140 948 H1780" stroke={BORDER} stroke-width="1" opacity="0.48"/>
   <g opacity={search}>
    <text x="350" y="153" font-family={MONO} font-size="27" fill={AQUA}>"TANTIVY-TURBO  /  SEARCH"</text>
    <text x="350" y="240" font-family={SANS} font-weight="600" font-size="72" fill={INK}>"Find what matters."</text>
    <rect x="350" y="292" width="1220" height="112" rx="24" fill={PANEL} stroke={BLUE} stroke-width="3" />
    <circle cx="413" cy="343" r="15" fill="none" stroke={AQUA} stroke-width="5" />
    <line x1="424" y1="355" x2="439" y2="370" stroke={AQUA} stroke-width="5" stroke-linecap="round" />
    <text x="475" y="363" font-family={SANS} font-size="42" fill={INK}>{&input[..n]}</text>
    <line x1="351" y1="405" x2={351.0+1218.0*trace} y2="405" stroke={AQUA} stroke-width="4" opacity="0.80" />
    <circle cx={351.0+1218.0*trace} cy="405" r="22" fill={BLUE} opacity="0.20" />
    {results}
    <text x="350" y="893" font-family={MONO} font-size="26" fill={DIM}>"ILLUSTRATIVE SEARCH"</text>
   </g>
   <g opacity={java}>
    <text x="160" y="150" font-family={MONO} font-size="27" fill={AQUA}>"APACHE LUCENE 10.4 / JAVA"</text>
    <text x="160" y="255" font-family={SANS} font-weight="600" font-size="70" fill={INK}>"The core behind search."</text>
    <text x="160" y="327" font-family={SANS} font-size="36" fill={DIM}>"Elasticsearch · OpenSearch · Solr"</text>
    <circle cx="520" cy="466" r="96" fill={PANEL} stroke={BLUE} stroke-width="3" />
    <text x="520" y="479" text-anchor="middle" font-family={MONO} font-size="28" fill={INK}>"LUCENE"</text>
    {networks}
    <g opacity={gate}>
     <text x="1130" y="478" font-family={SANS} font-weight="600" font-size="90" fill={INK}>"1.82–2.09×"</text>
     <text x="1135" y="546" font-family={SANS} font-size="45" fill={AQUA}>"faster"</text>
    </g>
    <text x="160" y="824" font-family={MONO} font-size="29" fill={DIM}>"20 searches · top 10 · older build · Oct 4"</text>
    <text x="160" y="869" font-family={MONO} font-size="25" fill={DIM}>"geometric mean · two process orders"</text>
   </g>
   <g opacity={port}>
    <text x="160" y="150" font-family={MONO} font-size="27" fill={AQUA}>"LUCENE-RS / RUST PORT"</text>
    <text x="160" y="255" font-family={SANS} font-weight="600" font-size="70" fill={INK}>"Less waiting."</text>
    <text x="160" y="310" font-family={SANS} font-size="29" fill={AQUA}>"Less time is better"</text>
    <text x="160" y="355" font-family={SANS} font-size="37" fill={DIM}>"lucene-rs"</text>
    <text x="160" y="538" font-family={SANS} font-size="37" fill={INK}>"tantivy-turbo"</text>
    <rect x="160" y="382" width="680" height="62" rx="31" fill={PANEL}/>
    <rect x="160" y="565" width="680" height="62" rx="31" fill={PANEL}/>
    <rect x="160" y="382" width={w1} height="62" rx="31" fill={DIM}/>
    <rect x="160" y="565" width={w2} height="62" rx="31" fill={BLUE}/>
    <circle cx={160.0+w2} cy="596" r="28" fill={BLUE} opacity="0.22"/>
    <circle cx={160.0+w2} cy="596" r="6" fill={AQUA}/>
    <text x="880" y="429" font-family={MONO} font-size="33" fill={DIM} opacity={score}>"168.5 µs"</text>
    <text x="880" y="612" font-family={MONO} font-size="33" fill={INK} opacity={score}>"136.2 µs"</text>
    <g opacity={score}>
     <text x="1250" y="470" font-family={SANS} font-weight="600" font-size="95" fill={INK}>"1.24×"</text>
     <text x="1255" y="535" font-family={SANS} font-size="42" fill={AQUA}>"faster"</text>
    </g>
    <text x="160" y="824" font-family={MONO} font-size="29" fill={DIM}>"1,201 searches · warm · 1 core · Oct 28"</text>
    <text x="160" y="869" font-family={MONO} font-size="25" fill={DIM}>"mean query medians · animated scale"</text>
   </g>
   <g opacity={proof}>
    <circle cx="960" cy="400" r={ring+42.0} fill={BLUE} opacity="0.08"/>
    <circle cx="960" cy="400" r={ring} fill={PANEL} stroke={BLUE} stroke-width="3"/>
    <path d="M 900 401 l 42 42 l 79 -93" fill="none" stroke={AQUA} stroke-width="14" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="215" stroke-dashoffset={215.0*(1.0-checked)}/>
    <text x="960" y="690" text-anchor="middle" font-family={SANS} font-weight="600" font-size="76" fill={INK}>"14,400 checks"</text>
    <text x="960" y="765" text-anchor="middle" font-family={SANS} font-size="40" fill={DIM}>"against a full scan"</text>
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
