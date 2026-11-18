use crate::domain::Granule;
use crate::providers::nasa::icesat2::pairing::pairs::{Candidate, Icesat2MatchKey};

pub(super) fn identity(name: &str, product: &str) -> Option<(Icesat2MatchKey, u8)> {
    let stem = name.strip_suffix(".h5")?;
    let fields: Vec<_> = stem.split('_').collect();
    if fields.len() != 5 || fields[0] != product {
        return None;
    }
    for (value, width) in fields[1..].iter().zip([14, 8, 3, 2]) {
        if value.len() != width || !value.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
    }
    chrono::NaiveDateTime::parse_from_str(fields[1], "%Y%m%d%H%M%S").ok()?;
    let reference_ground_track = fields[2][..4].parse().ok()?;
    let cycle = fields[2][4..6].parse().ok()?;
    let region = fields[2][6..].parse().ok()?;
    if !(1..=1387).contains(&reference_ground_track) || !(1..=14).contains(&region) {
        return None;
    }
    Some((
        Icesat2MatchKey {
            acquisition: fields[1].into(),
            reference_ground_track,
            cycle,
            region,
            version: fields[3].parse().ok()?,
        },
        fields[4].parse().ok()?,
    ))
}

pub(super) fn candidate(
    mut granule: Granule,
    product: &str,
) -> Option<(Icesat2MatchKey, Candidate)> {
    let mut selected = None;
    for asset in std::mem::take(&mut granule.assets) {
        let url = reqwest::Url::parse(&asset.url).ok()?;
        let Some(name) = url.path_segments().and_then(|mut parts| parts.next_back()) else {
            continue;
        };
        let Some((key, revision)) = identity(name, product) else {
            continue;
        };
        if let Some((previous_key, previous_revision, _)) = &selected {
            if previous_key != &key || previous_revision != &revision {
                return None;
            }
        } else {
            selected = Some((key, revision, asset));
        }
    }
    let (key, revision, asset) = selected?;
    granule.assets.push(asset);
    Some((key, Candidate { revision, granule }))
}
