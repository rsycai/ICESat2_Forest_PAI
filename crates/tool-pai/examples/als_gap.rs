use anyhow::{Context, Result, ensure};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use xt_core::geotiff::GeoTiffSampler;

#[derive(Parser)]
struct Args {
    directory: PathBuf,
    #[arg(long)]
    g0_raster: Option<PathBuf>,
}

#[derive(Deserialize)]
struct Index {
    #[serde(rename = "window")]
    windows: Vec<Window>,
}

#[derive(Deserialize)]
struct Window {
    #[serde(rename = "@file")]
    file: String,
    #[serde(rename = "@beam")]
    beam: String,
    #[serde(rename = "@segment_id_beg")]
    segment_id_beg: u32,
    #[serde(rename = "@segment_id_end")]
    segment_id_end: u32,
    #[serde(rename = "@longitude")]
    longitude: f64,
    #[serde(rename = "@latitude")]
    latitude: f64,
}

#[derive(Serialize)]
struct Row<'a> {
    file: &'a str,
    beam: &'a str,
    segment_id_beg: u32,
    segment_id_end: u32,
    longitude: f64,
    latitude: f64,
    valid_pulses: usize,
    ground_pulses: usize,
    gap: Option<f64>,
    g0: f64,
    g0_source: &'static str,
    paie: Option<f64>,
}

fn count_pulses(path: &Path) -> Result<(usize, usize)> {
    let mut reader = las::Reader::from_path(path)?;
    ensure!(
        reader.header().point_format().has_gps_time,
        "Missing pulse GPS time"
    );
    let mut pulses = HashMap::new();
    for point in reader.points() {
        let p = point?;
        let class = u8::from(p.classification);
        if p.is_withheld
            || p.is_synthetic
            || !(2..=5).contains(&class)
            || p.return_number == 0
            || p.return_number > p.number_of_returns
        {
            continue;
        }
        let t = p
            .gps_time
            .filter(|t| t.is_finite())
            .context("Valid return lacks a finite GPS time")?;
        let key = (p.point_source_id, p.scanner_channel, t.to_bits());
        *pulses.entry(key).or_insert(false) |= class == 2;
    }
    Ok((
        pulses.len(),
        pulses.values().filter(|&&ground| ground).count(),
    ))
}

fn main() -> Result<()> {
    let args = Args::parse();
    let index = args.directory.join("index.xml");
    let index: Index = quick_xml::de::from_reader(BufReader::new(
        File::open(&index).with_context(|| format!("Read {}", index.display()))?,
    ))?;
    ensure!(!index.windows.is_empty(), "Manifest contains no ALS windows");
    let raster = match args.g0_raster {
        Some(path) => path,
        None => tool_pai::PaiConfig::new()?.g0_raster,
    };
    let coords: Vec<_> = index
        .windows
        .iter()
        .map(|w| (w.longitude, w.latitude))
        .collect();
    let samples = GeoTiffSampler::open(raster)?.sample_many::<f32>(&coords)?;
    let mut temp = tempfile::NamedTempFile::new_in(&args.directory)?;
    {
        let mut csv = csv::Writer::from_writer(temp.as_file_mut());
        for (w, sample) in index.windows.iter().zip(samples) {
            let (valid_pulses, ground_pulses) = count_pulses(&args.directory.join(&w.file))
                .with_context(|| format!("Summarize {}", w.file))?;
            let (g0, g0_source) = match sample.filter(|&g| g > 0.0 && g <= 1.0) {
                Some(g) => (f64::from(g), "raster"),
                None => (0.5, "fallback"),
            };
            let gap = (valid_pulses > 0).then(|| ground_pulses as f64 / valid_pulses as f64);
            let paie = gap
                .filter(|&p| p > 0.0)
                .map(|p| xt_laipath::calc_eff_pai(p, 0.0, g0))
                .transpose()?;
            csv.serialize(Row {
                file: &w.file,
                beam: &w.beam,
                segment_id_beg: w.segment_id_beg,
                segment_id_end: w.segment_id_end,
                longitude: w.longitude,
                latitude: w.latitude,
                valid_pulses,
                ground_pulses,
                gap,
                g0,
                g0_source,
                paie,
            })?;
        }
        csv.flush()?;
    }
    temp.persist(args.directory.join("als_gap_paie.csv"))?;
    Ok(())
}
