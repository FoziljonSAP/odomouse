//! Replays a recorded input stream (fixtures/cross.json, made with the first,
//! JavaScript version of the tracker) and checks the Rust core ends up with
//! the same days and summaries, so old history files keep their meaning.

use odomouse_core::day::{aggregate, normalize_day, summarize};
use odomouse_core::displays::{build_model, infos_from_json};
use odomouse_core::json::{parse, Json};
use odomouse_core::time::Moment;
use odomouse_core::tracker::Tracker;

fn approx_eq(a: &Json, b: &Json, path: &str) {
    match (a, b) {
        (Json::Num(x), Json::Num(y)) => {
            let tol = 1e-9 * x.abs().max(y.abs()).max(1.0);
            assert!((x - y).abs() <= tol, "{}: {} != {}", path, x, y);
        }
        (Json::Arr(x), Json::Arr(y)) => {
            assert_eq!(x.len(), y.len(), "{}: length", path);
            for (i, (p, q)) in x.iter().zip(y).enumerate() {
                approx_eq(p, q, &format!("{}[{}]", path, i));
            }
        }
        (Json::Obj(x), Json::Obj(_)) => {
            for (k, v) in x {
                match b.get(k) {
                    Some(w) => approx_eq(v, w, &format!("{}.{}", path, k)),
                    None => assert!(matches!(v, Json::Null), "{}.{} missing on the JS side", path, k),
                }
            }
            for (k, _) in b.as_obj().unwrap() {
                assert!(a.get(k).is_some(), "{}.{} missing on the Rust side", path, k);
            }
        }
        _ => assert_eq!(a, b, "{}", path),
    }
}

#[test]
fn rust_core_matches_the_javascript_tracker() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cross.json")).unwrap();
    let fx = parse(&text).unwrap();
    let tz = fx.get("tzOffsetS").unwrap().as_f64().unwrap() as i32;
    let model = build_model(&infos_from_json(fx.get("displays").unwrap()), &[]);
    let events = fx.get("events").unwrap().as_arr().unwrap();
    let n = |e: &Json, i: usize| e.as_arr().unwrap()[i].as_f64().unwrap();
    let first = Moment::new(n(&events[0], 1) as i64, tz);
    let mut tr = Tracker::new(first, None);
    tr.set_displays(model);
    let mut days = Vec::new();
    for e in events {
        let a = e.as_arr().unwrap();
        let t = Moment::new(n(e, 1) as i64, tz);
        match a[0].as_str().unwrap() {
            "move" => tr.mouse_move(t, n(e, 2), n(e, 3)),
            "down" => tr.mouse_down(t, n(e, 2) as u32),
            "wheel" => tr.wheel(t, n(e, 2), n(e, 3), n(e, 4)),
            "keydown" => tr.key_down(t, n(e, 2) as u32, n(e, 3) as u32),
            "keyup" => tr.key_up(t, n(e, 2) as u32),
            "sample" => tr.sample_cursor(t, n(e, 2), n(e, 3)),
            "app" => tr.set_app(a[2].as_str()),
            "tick" => tr.check_rollover(t),
            other => panic!("unknown event {}", other),
        }
        days.extend(tr.take_finished());
    }
    days.push(tr.day.clone());

    let js_days = fx.get("days").unwrap().as_arr().unwrap();
    assert_eq!(days.len(), js_days.len());
    for (i, (r, j)) in days.iter().zip(js_days).enumerate() {
        let js = normalize_day(j, "");
        approx_eq(&r.to_json(), &js.to_json(), &format!("day{}", i));
        // the stream must actually exercise everything
        assert!(r.keystrokes > 100.0 && r.mouse_mm > 1000.0 && !r.shortcuts.is_empty() && !r.apps.is_empty());
        approx_eq(&summarize(r), &fx.get("summaries").unwrap().as_arr().unwrap()[i], &format!("summary{}", i));
    }
    approx_eq(&summarize(&aggregate(days.iter())), fx.get("aggregate").unwrap(), "aggregate");
}
