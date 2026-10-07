//! Screen geometry -> real-world distance. Cursor positions arrive in the
//! OS's global coordinate space (points on macOS, physical or scaled pixels
//! on Windows/Linux, whatever the shell reports: it only has to use the same
//! space for display bounds and cursor events). For each display we need
//! "millimetres per unit". Source priority:
//!   1. 'diagonal' – the user typed the screen diagonal in Settings
//!   2. 'hardware' – physical size from EDID, if plausible
//!   3. 'estimate' – typical density for that kind of display

use crate::json::Json;

pub const MM_PER_INCH: f64 = 25.4;
/// One step longer than this is a cursor warp, not motion.
pub const MAX_STEP_POINTS: f64 = 2500.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// What the native shell reports about a display.
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub id: String,
    pub label: Option<String>,
    pub internal: bool,
    pub scale_factor: f64,
    pub bounds: Rect,
    /// Physical size from EDID (0 when unknown).
    pub width_mm: f64,
    pub height_mm: f64,
    /// Units per inch when the shell knows the scaling but not the size
    /// (Windows/Linux report pixels; 0 = use the macOS point estimates).
    pub estimate_ppi: f64,
}

#[derive(Debug, Clone)]
pub struct Display {
    pub id: String,
    pub label: String,
    /// The shell gave no name: `label` is the generic "Ichki ekran" /
    /// "Tashqi monitor", shown translated.
    pub generic_label: bool,
    pub internal: bool,
    pub scale_factor: f64,
    pub bounds: Rect,
    pub mm_per_point: f64,
    pub source: &'static str,
    pub width_mm: f64,
    pub height_mm: f64,
    pub diagonal_inches: f64,
    pub points_per_inch: f64,
}

pub fn is_plausible_physical_size(w: f64, h: f64, b: &Rect) -> bool {
    if b.width <= 0.0 || b.height <= 0.0 {
        return false;
    }
    if !((150.0..=2500.0).contains(&w) && (90.0..=1600.0).contains(&h)) {
        return false;
    }
    let pixel_aspect = b.width / b.height;
    let (w, h) = if (w > h) != (b.width > b.height) { (h, w) } else { (w, h) };
    ((w / h) - pixel_aspect).abs() / pixel_aspect <= 0.08
}

pub fn mm_per_point_from_diagonal(b: &Rect, inches: f64) -> f64 {
    inches * MM_PER_INCH / b.width.hypot(b.height)
}

pub fn estimate_mm_per_point(d: &DisplayInfo) -> f64 {
    let ppi = if d.estimate_ppi > 0.0 {
        d.estimate_ppi
    } else if d.internal {
        125.0
    } else if d.scale_factor >= 2.0 {
        109.0
    } else {
        92.0
    };
    MM_PER_INCH / ppi
}

/// `overrides`: (display id, diagonal inches) from settings.
pub fn build_model(infos: &[DisplayInfo], overrides: &[(String, f64)]) -> Vec<Display> {
    infos
        .iter()
        .filter(|d| d.bounds.width > 0.0 && d.bounds.height > 0.0)
        .map(|d| {
            let b = d.bounds;
            let ov = overrides.iter().find(|o| o.0 == d.id).map(|o| o.1).unwrap_or(0.0);
            let (mm, source) = if (5.0..=120.0).contains(&ov) {
                (mm_per_point_from_diagonal(&b, ov), "diagonal")
            } else if is_plausible_physical_size(d.width_mm, d.height_mm, &b) {
                (d.width_mm.max(d.height_mm) / b.width.max(b.height), "hardware")
            } else {
                (estimate_mm_per_point(d), "estimate")
            };
            let (wmm, hmm) = (b.width * mm, b.height * mm);
            Display {
                id: d.id.clone(),
                label: d.label.clone().filter(|l| !l.is_empty()).unwrap_or_else(|| {
                    if d.internal { "Ichki ekran" } else { "Tashqi monitor" }.to_string()
                }),
                generic_label: d.label.as_deref().map_or(true, str::is_empty),
                internal: d.internal,
                scale_factor: if d.scale_factor > 0.0 { d.scale_factor } else { 1.0 },
                bounds: b,
                mm_per_point: mm,
                source,
                width_mm: wmm,
                height_mm: hmm,
                diagonal_inches: wmm.hypot(hmm) / MM_PER_INCH,
                points_per_inch: MM_PER_INCH / mm,
            }
        })
        .collect()
}

fn contains(b: &Rect, x: f64, y: f64) -> bool {
    x >= b.x && x < b.x + b.width && y >= b.y && y < b.y + b.height
}

fn distance_to_rect(b: &Rect, x: f64, y: f64) -> f64 {
    let dx = (b.x - x).max(0.0).max(x - (b.x + b.width - 1.0));
    let dy = (b.y - y).max(0.0).max(y - (b.y + b.height - 1.0));
    dx.hypot(dy)
}

/// Index of the display containing the point, or the nearest one.
pub fn locate(model: &[Display], x: f64, y: f64) -> Option<usize> {
    let mut best = None;
    let mut best_dist = f64::INFINITY;
    for (i, d) in model.iter().enumerate() {
        if contains(&d.bounds, x, y) {
            return Some(i);
        }
        let dist = distance_to_rect(&d.bounds, x, y);
        if dist < best_dist {
            best_dist = dist;
            best = Some(i);
        }
    }
    best
}

/// Real-world length of one cursor step in mm; 0 for warps.
pub fn step_mm(model: &[Display], x0: f64, y0: f64, x1: f64, y1: f64) -> f64 {
    let points = (x1 - x0).hypot(y1 - y0);
    if points == 0.0 || points > MAX_STEP_POINTS || !points.is_finite() {
        return 0.0;
    }
    match (locate(model, x0, y0), locate(model, x1, y1)) {
        (Some(a), Some(b)) => {
            let scale = if a == b {
                model[a].mm_per_point
            } else {
                (model[a].mm_per_point + model[b].mm_per_point) / 2.0
            };
            points * scale
        }
        _ => 0.0,
    }
}

/// Position inside its display as 0..1 fractions (for the cursor heatmap).
pub fn normalized_position(model: &[Display], x: f64, y: f64) -> Option<(usize, f64, f64)> {
    let i = locate(model, x, y)?;
    let b = &model[i].bounds;
    let fx = ((x - b.x) / b.width).clamp(0.0, 0.999999);
    let fy = ((y - b.y) / b.height).clamp(0.0, 0.999999);
    Some((i, fx, fy))
}

/// Shell input: [{id, label?, internal, scaleFactor, bounds:{x,y,width,height}, widthMm?, heightMm?, estimatePpi?}]
pub fn infos_from_json(j: &Json) -> Vec<DisplayInfo> {
    let num = |v: Option<&Json>| v.and_then(Json::as_f64).filter(|n| n.is_finite()).unwrap_or(0.0);
    j.as_arr()
        .map(|arr| {
            arr.iter()
                .map(|d| {
                    let b = d.get("bounds");
                    let id = match d.get("id") {
                        Some(Json::Str(s)) => s.clone(),
                        Some(Json::Num(n)) => format!("{}", *n as i64),
                        _ => String::new(),
                    };
                    DisplayInfo {
                        id,
                        label: d.get("label").and_then(Json::as_str).map(str::to_string),
                        internal: d.get("internal").and_then(Json::as_bool).unwrap_or(false),
                        scale_factor: num(d.get("scaleFactor")).max(1.0),
                        bounds: Rect {
                            x: num(b.and_then(|b| b.get("x"))),
                            y: num(b.and_then(|b| b.get("y"))),
                            width: num(b.and_then(|b| b.get("width"))),
                            height: num(b.and_then(|b| b.get("height"))),
                        },
                        width_mm: num(d.get("widthMm")),
                        height_mm: num(d.get("heightMm")),
                        estimate_ppi: num(d.get("estimatePpi")),
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The shape the settings page shows.
pub fn public_json(model: &[Display]) -> Json {
    public_json_in(model, crate::i18n::Lang::Uz)
}

pub fn public_json_in(model: &[Display], lang: crate::i18n::Lang) -> Json {
    Json::Arr(
        model
            .iter()
            .map(|d| {
                let label = if !d.generic_label {
                    d.label.as_str()
                } else if d.internal {
                    lang.pick("Ichki ekran", "Built-in display", "Встроенный экран")
                } else {
                    lang.pick("Tashqi monitor", "External monitor", "Внешний монитор")
                };
                Json::obj()
                    .set("id", d.id.as_str())
                    .set("label", label)
                    .set("internal", d.internal)
                    .set("source", d.source)
                    .set("width", d.bounds.width)
                    .set("height", d.bounds.height)
                    .set("diagonalInches", d.diagonal_inches)
                    .set("pointsPerInch", d.points_per_inch)
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(id: &str, x: f64, w: f64, h: f64, internal: bool, mm: (f64, f64)) -> DisplayInfo {
        DisplayInfo {
            id: id.into(),
            label: None,
            internal,
            scale_factor: 2.0,
            bounds: Rect { x, y: 0.0, width: w, height: h },
            width_mm: mm.0,
            height_mm: mm.1,
            estimate_ppi: 0.0,
        }
    }

    #[test]
    fn hardware_size_beats_estimate_and_override_beats_both() {
        // MacBook Air 13.6": 1470x956 pt, 302x196 mm
        let air = info("1", 0.0, 1470.0, 956.0, true, (302.0, 196.0));
        let m = build_model(&[air.clone()], &[]);
        assert_eq!(m[0].source, "hardware");
        assert!((m[0].mm_per_point - 302.0 / 1470.0).abs() < 1e-12);
        assert!((m[0].diagonal_inches - 14.18).abs() < 0.1);
        let m = build_model(&[air.clone()], &[("1".into(), 13.6)]);
        assert_eq!(m[0].source, "diagonal");
        assert!((m[0].diagonal_inches - 13.6).abs() < 1e-9);
        // missing EDID size (0x0) and a 4:3 size on a 16:9 grid are both ignored
        let odd = info("3", 0.0, 1920.0, 1080.0, false, (400.0, 300.0));
        assert_eq!(build_model(&[odd], &[])[0].source, "estimate");
        let tv = info("2", 0.0, 1920.0, 1080.0, false, (0.0, 0.0));
        let m = build_model(&[tv], &[]);
        assert_eq!(m[0].source, "estimate");
        assert!((m[0].points_per_inch - 109.0).abs() < 1e-9);
        assert_eq!(m[0].label, "Tashqi monitor");
    }

    #[test]
    fn rotated_screen_and_aspect_check() {
        let b = Rect { x: 0.0, y: 0.0, width: 1080.0, height: 1920.0 };
        assert!(is_plausible_physical_size(527.0, 296.0, &b));
        assert!(!is_plausible_physical_size(527.0, 400.0, &b));
        assert!(!is_plausible_physical_size(0.0, 0.0, &b));
    }

    #[test]
    fn steps_warps_and_crossing_monitors() {
        let a = info("1", 0.0, 1000.0, 800.0, true, (0.0, 0.0));
        let mut b = info("2", 1000.0, 1000.0, 800.0, false, (0.0, 0.0));
        b.scale_factor = 1.0;
        let m = build_model(&[a, b], &[]);
        let s1 = step_mm(&m, 0.0, 0.0, 3.0, 4.0);
        assert!((s1 - 5.0 * 25.4 / 125.0).abs() < 1e-9);
        let cross = step_mm(&m, 995.0, 10.0, 1005.0, 10.0);
        assert!((cross - 10.0 * (25.4 / 125.0 + 25.4 / 92.0) / 2.0).abs() < 1e-9);
        assert_eq!(step_mm(&m, 0.0, 0.0, 3000.0, 0.0), 0.0);
        assert_eq!(step_mm(&[], 0.0, 0.0, 3.0, 0.0), 0.0);
        let (i, fx, fy) = normalized_position(&m, 1500.0, 400.0).unwrap();
        assert_eq!(i, 1);
        assert!((fx - 0.5).abs() < 1e-9 && (fy - 0.5).abs() < 1e-9);
        // off-screen point snaps to the nearest display
        assert_eq!(locate(&m, -50.0, 10.0), Some(0));
    }
}
