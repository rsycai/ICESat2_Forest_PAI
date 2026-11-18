use crate::Granule;

pub(crate) fn acquisition(granule: &Granule) -> Option<String> {
    if let Some(time) = granule
        .acquired_at
        .as_ref()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
    {
        return Some(
            time.with_timezone(&chrono::Utc)
                .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
        );
    }
    let url_name = granule
        .assets
        .first()
        .and_then(|a| reqwest::Url::parse(&a.url).ok())
        .and_then(|u| {
            u.path_segments()
                .and_then(|mut p| p.next_back())
                .map(str::to_owned)
        });
    for name in std::iter::once(granule.title.as_str()).chain(url_name.as_deref()) {
        let timestamp = if name.starts_with("ATL") {
            name.split('_')
                .nth(1)
                .filter(|v| v.len() == 14)
                .and_then(|v| chrono::NaiveDateTime::parse_from_str(v, "%Y%m%d%H%M%S").ok())
        } else if name.starts_with("GEDI") {
            name.split('_')
                .find(|v| v.len() == 13 && v.bytes().all(|b| b.is_ascii_digit()))
                .and_then(|v| chrono::NaiveDateTime::parse_from_str(v, "%Y%j%H%M%S").ok())
        } else {
            None
        };
        if let Some(time) = timestamp {
            return Some(
                time.and_utc()
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            );
        }
    }
    None
}
