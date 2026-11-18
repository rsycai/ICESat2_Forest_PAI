use crate::{
    calculation::{projection, segment},
    model::{BEAMS, Location, Row},
};
use anyhow::{Result, ensure};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

#[derive(bincode::Decode)]
struct ArchivedSegment {
    begin: i32,
    end: i32,
    time: i64,
    lon: f32,
    lat: f32,
    distance_scale: f32,
    distance_offset: f32,
    height_scale: f32,
    height_offset: f32,
    distances: Vec<u16>,
    heights: Vec<u16>,
}

pub(crate) fn read(
    path: &Path,
    raster: &(),
) -> Result<BTreeMap<String, Vec<Row>>> {
    let mut input = BufReader::new(brotli_decompressor::Decompressor::new(
        File::open(path)?,
        65536,
    ));
    let cfg = bincode::config::standard().with_limit::<16777216>();
    let count: u64 = bincode::decode_from_std_read(&mut input, cfg)?;
    ensure!(count <= 6, "Invalid BR beam count");
    let mut out = BTreeMap::new();
    for _ in 0..count {
        let name: String = bincode::decode_from_std_read(&mut input, cfg)?;
        ensure!(
            BEAMS.contains(&name.as_str()) && !out.contains_key(&name),
            "Unknown or duplicate beam {name}"
        );
        let n: u64 = bincode::decode_from_std_read(&mut input, cfg)?;
        let mut results = Vec::new();
        let mut remaining = n;
        while remaining > 0 {
            let count = remaining.min(256);
            let batch = (0..count)
                .map(|_| bincode::decode_from_std_read(&mut input, cfg))
                .collect::<Result<Vec<ArchivedSegment>, _>>()?;
            let g0 = projection::sample(raster, batch.iter().map(|a| (a.lon, a.lat)))?;
            remaining -= count;
            for (a, g) in batch.into_iter().zip(g0) {
                ensure!(
                    a.distances.len() == a.heights.len(),
                    "{name}: inconsistent segment photon column lengths"
                );
                let samples = a.distances.into_iter().zip(a.heights).map(|(x, h)| {
                    (
                        f64::from(f32::from(x) * a.distance_scale + a.distance_offset),
                        f32::from(h) * a.height_scale + a.height_offset,
                    )
                });
                results.push(segment(
                    Location {
                        begin: a.begin,
                        end: a.end,
                        time: a.time as f64,
                        lon: a.lon,
                        lat: a.lat,
                    },
                    samples,
                    g,
                ));
            }
        }
        out.insert(name, results);
    }
    ensure!(input.read(&mut [0])? == 0, "Trailing data in BR archive");
    Ok(out)
}

