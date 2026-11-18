use chrono::{NaiveDate, NaiveDateTime};

pub(crate) struct Granule<'a> {
    pub id: String,
    pub date: NaiveDate,
    pub release: &'a str,
}

impl<'a> Granule<'a> {
    pub fn suffix(stem: &str) -> Option<&str> {
        ["ATL03_08_", "ATL00_", "ATL77_", "PAI_"]
            .iter()
            .find_map(|prefix| stem.strip_prefix(prefix))
    }

    pub fn parse(stem: &'a str) -> Option<Self> {
        let suffix = Self::suffix(stem)?;
        let mut parts = suffix.split('_');
        let mut fields = Vec::with_capacity(5);
        for width in [14, 8, 3, 2] {
            let value = parts.next()?;
            if value.len() != width || !value.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            fields.push(value);
        }
        if let Some(revision) = parts
            .next()
            .filter(|s| s.len() == 2 && s.bytes().all(|b| b.is_ascii_digit()))
        {
            fields.push(revision);
        }
        Some(Self {
            id: format!("ATL03_08_{}", fields.join("_")),
            date: NaiveDateTime::parse_from_str(fields[0], "%Y%m%d%H%M%S")
                .ok()?
                .date(),
            release: fields[2],
        })
    }
}
