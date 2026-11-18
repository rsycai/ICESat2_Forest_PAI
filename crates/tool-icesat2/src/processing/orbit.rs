use crate::processing::error::bad;
use crate::processing::reading::read1;
use hdf5_pure::File;
use xt_error::Result;

pub(super) fn verify_orbit(a: &File, b: &File, key: &str) -> Result<()> {
    let rgt_a = read1::<i16>(a, "orbit_info/rgt")?;
    let rgt_b = read1::<i16>(b, "orbit_info/rgt")?;
    let cycle_a = read1::<i8>(a, "orbit_info/cycle_number")?;
    let cycle_b = read1::<i8>(b, "orbit_info/cycle_number")?;
    let orbit_a = read1::<u16>(a, "orbit_info/orbit_number")?;
    let orbit_b = read1::<u16>(b, "orbit_info/orbit_number")?;
    let orbit_token = key.split('_').nth(1).ok_or_else(|| bad("Missing orbit identifier"))?;
    let expected_rgt = orbit_token[..4]
        .parse::<i16>()
        .map_err(|_| bad("Invalid RGT"))?;
    let expected_cycle = orbit_token[4..6]
        .parse::<i8>()
        .map_err(|_| bad("Invalid cycle"))?;
    if rgt_a.len() != 1
        || cycle_a.len() != 1
        || orbit_a.len() != 1
        || rgt_a != rgt_b
        || cycle_a != cycle_b
        || orbit_a != orbit_b
        || rgt_a[0] != expected_rgt
        || cycle_a[0] != expected_cycle
    {
        return Err(bad("Orbit metadata does not match the filename or the other product"));
    }
    Ok(())
}
