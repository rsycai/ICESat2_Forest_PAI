use crate::error::invalid;
use xt_error::Result;

pub fn pair_stem(atl03: &str, atl08: &str) -> Result<String> {
    fn parts<'a>(name: &'a str, product: &str) -> Result<Vec<&'a str>> {
        let values = name
            .strip_suffix(".h5")
            .ok_or_else(|| invalid("Original file must have a .h5 extension"))?
            .split('_')
            .collect::<Vec<_>>();
        if values.len() != 5
            || values[0] != product
            || values[1..]
                .iter()
                .zip([14, 8, 3, 2])
                .any(|(v, n)| v.len() != n || !v.bytes().all(|c| c.is_ascii_digit()))
        {
            return Err(invalid("Invalid original ICESat-2 filename format"));
        }
        Ok(values)
    }
    let a = parts(atl03, "ATL03")?;
    let b = parts(atl08, "ATL08")?;
    if a[1..4] != b[1..4] {
        return Err(invalid("ATL03 and ATL08 observation or release mismatch"));
    }
    let revision = if a[4] == b[4] {
        a[4].to_string()
    } else {
        format!("{}_{}", a[4], b[4])
    };
    Ok(format!("ATL03_08_{}_{}", a[1..4].join("_"), revision))
}
