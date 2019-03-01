use serde_json::{Map, Value};

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process::exit;

mod gpx;
mod twelvebit;

fn main() {
    let mut input = String::new();

    std::env::args_os().nth(1)
        .as_ref()
        .map(File::open)
        .unwrap_or_else(|| {
            eprintln!("usage: {} <file.trk>", std::env::args().next().unwrap());
            exit(2);
        })
        .unwrap_or_else(|e| {
            eprintln!("Error opening input: {}", e);
            exit(2);
        })
        .read_to_string(&mut input)
        .unwrap_or_else(|e| {
            eprintln!("Error reading input: {}", e);
            exit(2);
        });

    let prefix = "onTrackReady(";
    let segments = if input.starts_with(prefix) {
        decode_json(&input[prefix.len() .. input.len() - 1])
    } else {
        eprintln!("Input doesn't look right; it doesn't start with \"{}\".", prefix);
        exit(2);
    };

    let mut ordered_segments = segments.into_iter()
        .map(|(_id, points)| points)
        .collect::<Vec<Vec<Point>>>();

    ordered_segments.sort_by_key(|points| points[0].time.floor() as i64);
    
    gpx::write_gpx(
        std::io::stdout(),
        ordered_segments.iter()
            .map(|points| points.as_slice())
        ).unwrap();
}

#[derive(Debug, Default)]
pub struct Point {
    alt: f64,
    lat: f64,
    lon: f64,
    speed: f64,
    time: f64,
}

fn decode_json(s: &str) -> HashMap<String, Vec<Point>> {
    let mut result = HashMap::new();
    let root = serde_json::from_str::<Value>(s).unwrap();
    for (track_id, track) in root.as_object().unwrap() {
        result.insert(track_id.to_owned(), decode_track(track.as_object().unwrap()));
    }
    result
}

fn decode_track(track: &Map<String, Value>) -> Vec<Point> {
    let mut all_data = HashMap::<String, Vec<f64>>::new();
    for (field, point_data) in track["data"].as_object().unwrap() {
        all_data.insert(
            field.to_owned(),
            decode_values(point_data.as_object().unwrap()));

    }
    let num = track["size"].as_u64().unwrap() as usize;
    let mut points = Vec::with_capacity(num);
    points.resize_with(num, Point::default);
    for field in ["alt","lat","lon","speed","time"].iter().cloned() {
        if all_data[field].len() != num {
            eprintln!("wrong number of points for {} segment: {} vs expected {}",
                      field, all_data[field].len(), num);
        }
        for (point, data) in points.iter_mut().zip(all_data[field].iter().cloned()) {
            match field {
                "alt" => { point.alt = data; }
                "lat" => { point.lat = data; }
                "lon" => { point.lon = data; }
                "speed" => { point.speed = data; }
                "time" => { point.time = data; }
                _ => unreachable!()
            }
        }
    }
    points
}

fn decode_values(data: &Map<String, Value>) -> Vec<f64> {
    let mut values = vec![];

    for seg in data["segments"].as_array().unwrap() {
        if seg["type"].as_str() != Some("double") {
            unimplemented!("data type {:?}", data["type"]);
        }
        
        let base = seg["base"].as_f64().unwrap();
        let size = seg["size"].as_i64().unwrap();

        match seg["encoding"].as_str().unwrap() {
            "freq" => {
                let step = seg["step"].as_f64().unwrap();
                let mut val = base;
                for _ in 0 .. size {
                    values.push(val);
                    val += step;
                }
            }
            "base64/diff" => {
                let bitwidth = seg["bitwidth"].as_i64().unwrap();
                let factor = seg["factor"].as_f64().unwrap();

                // there's also this but it appears to always be set to true, so...
                //let signed = seg["signed"].as_bool().unwrap();

                let data: Vec<u8> = base64::decode(seg["data"].as_str().unwrap()).unwrap();
                values.push(base);
                let mut last = base;
                match bitwidth {
                    8 => {
                        for byte in data {
                            let mut val = f64::from(byte);
                            if val >= 128. {
                                val -= 256.;
                            }
                            val *= factor;
                            val += last;
                            values.push(val);
                            last = val;
                        }
                    }
                    12 => {
                        for twelve in twelvebit::TwelveBits::new(data.into_iter()) {
                            let mut val = f64::from(twelve);
                            if val >= 2048. {
                                val -= 4096.;
                            }
                            val *= factor;
                            val += last;
                            values.push(val);
                            last = val;
                        }
                    }
                    _ => unimplemented!("base64/diff bitwidth {}", bitwidth)
                }
            }
            _ => unimplemented!("encoding {:?}", seg["encoding"])
        }
    }

    values
}
