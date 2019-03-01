use super::Point;

use std::io::{self, Write};

pub fn write_gpx<'a>(mut w: impl Write, segments: impl Iterator<Item=&'a [Point]>) -> io::Result<()> {
    writeln!(w, r#"<?xml version="1.0" encoding="utf-8"?>
<gpx xmlns="http://www.topografix.com/GPX/1/1" version="1.1" creator="alpinereplay-rs/1">"#)?;
    for seg in segments {
        writeln!(w, "<trk><trkseg>")?;
        for point in seg {
            let time = chrono::offset::TimeZone::timestamp(&chrono::Utc,
                point.time.floor() as i64,
                (point.time.fract() * 1_000_000_000.).floor() as u32);
            writeln!(w, r#"<trkpt lat="{}" lon="{}"><ele>{}</ele><time>{}</time></trkpt>"#,
                point.lat,
                point.lon,
                point.alt,
                time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))?;
        }
        writeln!(w, "</trkseg></trk>")?;
    }
    writeln!(w, "</gpx>")?;
    Ok(())
}
