use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::io::Cursor;

pub const EARTH_RADIUS_M: f64 = 6_371_000.0;
const MOVING_MIN_DELTA_M: f64 = 1.0;
const ELEVATION_MIN_DELTA_M: f64 = 1.0;

#[derive(Debug, Clone, Copy, Default)]
pub struct TrackPoint {
    pub time: Option<i64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub ele: Option<f64>,
    pub hr: Option<u16>,
}

#[derive(Debug, Default)]
pub struct Activity {
    pub points: Vec<TrackPoint>,
    pub total_time_s: Option<f64>,
    pub total_distance_m: Option<f64>,
    pub avg_hr: Option<u16>,
    pub max_hr: Option<u16>,
}

#[derive(Debug, Default)]
pub struct Metrics {
    pub distance_km: f64,
    pub moving_time_s: u32,
    pub elevation_m: Option<u32>,
    pub avg_hr: Option<u16>,
    pub max_hr: Option<u16>,
    pub start_utc: Option<i64>,
}

pub fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let dphi = (lat2 - lat1).to_radians();
    let dlambda = (lon2 - lon1).to_radians();
    let a = (dphi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (dlambda / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * a.sqrt().asin()
}

pub fn parse_gpx(bytes: &[u8]) -> Result<Activity> {
    let gpx = gpx::read(Cursor::new(bytes)).context("failed to parse GPX")?;
    let mut activity = Activity::default();
    for track in &gpx.tracks {
        for segment in &track.segments {
            for wpt in &segment.points {
                let p = wpt.point();
                activity.points.push(TrackPoint {
                    time: wpt.time.map(|t| {
                        let odt: time::OffsetDateTime = t.into();
                        odt.unix_timestamp()
                    }),
                    lat: Some(p.y()),
                    lon: Some(p.x()),
                    ele: wpt.elevation,
                    hr: None,
                });
            }
        }
    }
    Ok(activity)
}

fn local_name(qname: &[u8]) -> String {
    let idx = qname.iter().rposition(|&b| b == b':');
    match idx {
        Some(i) => String::from_utf8_lossy(&qname[i + 1..]).into_owned(),
        None => String::from_utf8_lossy(qname).into_owned(),
    }
}

fn parse_xml_time(s: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(s.trim())
        .ok()
        .map(|dt| dt.with_timezone(&Utc).timestamp())
}

pub fn parse_tcx(bytes: &[u8]) -> Result<Activity> {
    let text = String::from_utf8_lossy(bytes);
    let mut reader = quick_xml::Reader::from_str(&text);
    reader.config_mut().trim_text(true);

    let mut activity = Activity::default();
    let mut stack: Vec<String> = Vec::new();
    let mut current: Option<TrackPoint> = None;

    loop {
        use quick_xml::events::Event;
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = local_name(e.name().as_ref());
                if name == "Trackpoint" {
                    current = Some(TrackPoint::default());
                }
                stack.push(name);
            }
            Ok(Event::End(_)) => {
                let closing = stack.pop();
                if closing.as_deref() == Some("Trackpoint") {
                    if let Some(p) = current.take() {
                        activity.points.push(p);
                    }
                }
            }
            Ok(Event::Text(t)) => {
                let value = match t.xml_content() {
                    Ok(v) => v.into_owned(),
                    Err(_) => continue,
                };
                assign_text(&stack, &value, current.as_mut(), &mut activity);
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(anyhow!("failed to parse TCX: {e}")),
        }
    }
    Ok(activity)
}

fn assign_text(
    stack: &[String],
    value: &str,
    point: Option<&mut TrackPoint>,
    activity: &mut Activity,
) {
    let parent = |n: usize| {
        stack
            .get(stack.len().saturating_sub(1 + n))
            .map(String::as_str)
    };
    let tag = parent(0).unwrap_or("");
    let grandparent = parent(1).unwrap_or("");
    match tag {
        "Time" => {
            if let Some(p) = point {
                p.time = parse_xml_time(value);
            }
        }
        "LatitudeDegrees" => {
            if let Some(p) = point {
                p.lat = value.trim().parse().ok();
            }
        }
        "LongitudeDegrees" => {
            if let Some(p) = point {
                p.lon = value.trim().parse().ok();
            }
        }
        "AltitudeMeters" => {
            if let Some(p) = point {
                p.ele = value.trim().parse().ok();
            }
        }
        "Value" => {
            let hr: Option<u16> = value.trim().parse().ok();
            if let (Some(p), Some(hr)) = (point, hr) {
                p.hr = Some(hr);
            } else if let Some(hr) = hr {
                match grandparent {
                    "AverageHeartRateBpm" => activity.avg_hr = Some(hr),
                    "MaximumHeartRateBpm" => activity.max_hr = Some(hr),
                    _ => {}
                }
            }
        }
        "TotalTimeSeconds" => activity.total_time_s = value.trim().parse().ok(),
        "DistanceMeters" => activity.total_distance_m = value.trim().parse().ok(),
        _ => {}
    }
}

#[derive(Debug, Deserialize)]
struct JsonPoint {
    lat: Option<f64>,
    lon: Option<f64>,
    #[serde(default)]
    ele: Option<f64>,
    #[serde(default)]
    time: Option<String>,
    #[serde(default)]
    hr: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct JsonActivity {
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    moving_time_s: Option<u32>,
    #[serde(default)]
    elapsed_time_s: Option<u32>,
    #[serde(default)]
    distance_km: Option<f64>,
    #[serde(default)]
    elevation_m: Option<u32>,
    #[serde(default)]
    avg_hr: Option<u16>,
    #[serde(default)]
    max_hr: Option<u16>,
    #[serde(default)]
    points: Vec<JsonPoint>,
}

pub struct JsonSummary {
    pub date_hint: Option<String>,
    pub elevation_hint: Option<u32>,
}

pub fn parse_json(bytes: &[u8]) -> Result<(Activity, JsonSummary)> {
    let parsed: JsonActivity =
        serde_json::from_slice(bytes).context("failed to parse JSON activity")?;
    let summary = JsonSummary {
        date_hint: parsed.date.clone(),
        elevation_hint: parsed.elevation_m,
    };
    let activity = Activity {
        total_time_s: parsed
            .moving_time_s
            .map(f64::from)
            .or(parsed.elapsed_time_s.map(f64::from)),
        total_distance_m: parsed.distance_km.map(|km| km * 1000.0),
        avg_hr: parsed.avg_hr,
        max_hr: parsed.max_hr,
        points: parsed
            .points
            .iter()
            .map(|p| TrackPoint {
                lat: p.lat,
                lon: p.lon,
                ele: p.ele,
                time: p.time.as_deref().and_then(parse_xml_time),
                hr: p.hr,
            })
            .collect(),
    };
    Ok((activity, summary))
}

pub fn compute_metrics(activity: &Activity, elevation_hint: Option<u32>) -> Metrics {
    let mut distance_m = 0.0;
    let mut moving_s: i64 = 0;
    let mut has_times = false;
    let mut elevation_gain = 0.0;
    let mut has_elevation = false;
    let mut hr_sum: u64 = 0;
    let mut hr_count: u64 = 0;
    let mut hr_max: u16 = 0;

    for p in &activity.points {
        if let Some(hr) = p.hr {
            hr_sum += hr as u64;
            hr_count += 1;
            hr_max = hr_max.max(hr);
        }
    }

    for w in activity.points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg_m = match (a.lat, a.lon, b.lat, b.lon) {
            (Some(lat1), Some(lon1), Some(lat2), Some(lon2)) => {
                let d = haversine_m(lat1, lon1, lat2, lon2);
                distance_m += d;
                Some(d)
            }
            _ => None,
        };
        if let (Some(ta), Some(tb)) = (a.time, b.time) {
            let dt = tb - ta;
            has_times = true;
            if dt > 0 && seg_m.unwrap_or(0.0) > MOVING_MIN_DELTA_M {
                moving_s += dt;
            }
        }
        if let (Some(ea), Some(eb)) = (a.ele, b.ele) {
            has_elevation = true;
            let de = eb - ea;
            if de >= ELEVATION_MIN_DELTA_M {
                elevation_gain += de;
            }
        }
    }

    let start_utc = activity.points.iter().find_map(|p| p.time);

    if distance_m == 0.0 {
        distance_m = activity.total_distance_m.unwrap_or(0.0);
    }
    if !has_times {
        moving_s = activity.total_time_s.unwrap_or(0.0) as i64;
    }

    let elevation_m = if has_elevation {
        Some(elevation_gain.round() as u32)
    } else {
        elevation_hint
    };

    Metrics {
        distance_km: (distance_m / 1000.0 * 100.0).round() / 100.0,
        moving_time_s: moving_s.max(0) as u32,
        elevation_m,
        avg_hr: if hr_count > 0 {
            Some((hr_sum as f64 / hr_count as f64).round() as u16)
        } else {
            activity.avg_hr
        },
        max_hr: if hr_count > 0 {
            Some(hr_max)
        } else {
            activity.max_hr
        },
        start_utc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn haversine_known_distances() {
        let one_deg_equator = haversine_m(0.0, 0.0, 0.0, 1.0);
        assert!(approx(one_deg_equator, 111_195.0, 500.0));
        let one_deg_meridian = haversine_m(55.0, 37.0, 56.0, 37.0);
        assert!(approx(one_deg_meridian, 111_195.0, 500.0));
        assert!(approx(haversine_m(10.0, 20.0, 10.0, 20.0), 0.0, 1e-9));
    }

    #[test]
    fn metrics_from_synthetic_track() {
        let mk = |lat: f64, ele: f64, t: i64| TrackPoint {
            time: Some(t),
            lat: Some(lat),
            lon: Some(37.0),
            ele: Some(ele),
            hr: None,
        };
        let activity = Activity {
            points: vec![
                mk(55.000, 100.0, 0),
                mk(55.001, 102.5, 60),
                mk(55.002, 102.4, 120),
                mk(55.003, 105.0, 180),
            ],
            ..Default::default()
        };
        let m = compute_metrics(&activity, None);
        assert!(approx(m.distance_km, 0.334, 0.005));
        assert_eq!(m.moving_time_s, 180);
        assert_eq!(m.elevation_m, Some(5));
        assert_eq!(m.avg_hr, None);
    }

    #[test]
    fn metrics_skip_stationary_segments() {
        let mk = |lat: f64, ele: f64, t: i64| TrackPoint {
            time: Some(t),
            lat: Some(lat),
            lon: Some(30.0),
            ele: Some(ele),
            hr: None,
        };
        let activity = Activity {
            points: vec![
                mk(55.000, 10.0, 0),
                mk(55.000, 10.0, 300),
                mk(55.002, 12.0, 600),
            ],
            ..Default::default()
        };
        let m = compute_metrics(&activity, None);
        assert_eq!(m.moving_time_s, 300);
        assert_eq!(m.elevation_m, Some(2));
    }

    #[test]
    fn metrics_fall_back_to_summary_without_points() {
        let activity = Activity {
            total_distance_m: Some(8020.0),
            total_time_s: Some(2700.0),
            avg_hr: Some(145),
            max_hr: Some(162),
            ..Default::default()
        };
        let m = compute_metrics(&activity, Some(32));
        assert!(approx(m.distance_km, 8.02, 1e-9));
        assert_eq!(m.moving_time_s, 2700);
        assert_eq!(m.elevation_m, Some(32));
        assert_eq!(m.avg_hr, Some(145));
        assert_eq!(m.max_hr, Some(162));
    }

    #[test]
    fn metrics_use_point_hr_over_summary() {
        let mk = |lat: f64, hr: u16, t: i64| TrackPoint {
            time: Some(t),
            lat: Some(lat),
            lon: Some(30.0),
            ele: None,
            hr: Some(hr),
        };
        let activity = Activity {
            points: vec![mk(55.000, 140, 0), mk(55.001, 160, 60)],
            avg_hr: Some(999),
            max_hr: Some(999),
            ..Default::default()
        };
        let m = compute_metrics(&activity, None);
        assert_eq!(m.avg_hr, Some(150));
        assert_eq!(m.max_hr, Some(160));
    }
}
