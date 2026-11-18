use crate::storage::files::append_suffix;
use hya_core::Scheduler;
use hya_net::manifest::{ChunkAlgo, ChunkList, ChunkVerifier, Manifest, ObjectMeta, Trust};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::{fs, io};

const CHUNK_SIZE: u64 = 256 * 1024;

pub(super) fn restore(
    path: &Path,
    size: u64,
    resume: bool,
    asset_key: &str,
    validator: Option<&str>,
    scheduler: &mut Scheduler,
) -> io::Result<Manifest> {
    let mut manifest = Manifest {
        hydra_manifest: 1,
        object: ObjectMeta {
            size,
            chunk_size: CHUNK_SIZE,
            digest: None,
            validator: validator.map(str::to_owned),
            url: Some(format!("sha256:{asset_key}")),
        },
        chunks: ChunkList {
            algo: ChunkAlgo::Sha256,
            digests: vec![String::new(); Manifest::chunk_count(size, CHUNK_SIZE)],
        },
        parity: None,
    };
    let saved = resume
        .then(|| fs::read_to_string(append_suffix(path, ".hya.json")).ok())
        .flatten()
        .and_then(|text| Manifest::parse(&text).ok());
    if let Some(saved) = saved.filter(|m| {
        m.object.size == size
            && m.object.chunk_size == CHUNK_SIZE
            && m.chunks.algo == ChunkAlgo::Sha256
            && validator.is_some()
            && m.matches_validator(validator)
            && m.object.url == manifest.object.url
    }) && let Ok(mut file) = File::open(path)
        && file.metadata()?.len() == size
    {
        let mut buffer = vec![0; CHUNK_SIZE as usize];
        let mut verifier = ChunkVerifier::new(saved.clone(), Trust::Advertised);
        for (i, digest) in saved
            .chunks
            .digests
            .iter()
            .enumerate()
            .filter(|(_, d)| !d.is_empty())
        {
            let (lo, hi) = saved.span(i);
            file.seek(SeekFrom::Start(lo))?;
            file.read_exact(&mut buffer[..(hi - lo) as usize])?;
            if verifier.write(lo, &buffer[..(hi - lo) as usize]).is_empty() {
                scheduler.mark_done(lo, hi);
                manifest.chunks.digests[i] = digest.clone();
            }
        }
    }
    Ok(manifest)
}

pub(super) fn save(path: &Path, ranges: &[(u64, u64)], manifest: &mut Manifest) -> io::Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let mut buffer = vec![0; CHUNK_SIZE as usize];
    let mut changed = false;
    for i in 0..manifest.chunks.digests.len() {
        if !manifest.chunks.digests[i].is_empty() {
            continue;
        }
        let (lo, hi) = manifest.span(i);
        if !ranges.iter().any(|&(a, b)| a <= lo && b >= hi) {
            continue;
        }
        file.seek(SeekFrom::Start(lo))?;
        file.read_exact(&mut buffer[..(hi - lo) as usize])?;
        manifest.chunks.digests[i] = manifest.chunks.algo.hash(&buffer[..(hi - lo) as usize]);
        changed = true;
    }
    if changed {
        file.sync_data()?;
        let sidecar = append_suffix(path, ".hya.json");
        let temporary = append_suffix(&sidecar, ".tmp");
        fs::write(&temporary, manifest.to_json())?;
        OpenOptions::new()
            .write(true)
            .open(&temporary)?
            .sync_all()?;
        fs::rename(&temporary, &sidecar)?;
    }
    Ok(())
}
