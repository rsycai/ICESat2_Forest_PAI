use crate::output::attributes::error::h5;
use hdf5_pure::{Dataset, H5Element};
use xt_error::Result;

pub(super) trait Rows: H5Element + Copy {
    fn rows(d: &Dataset, start: u64, count: u64) -> Result<Vec<Self>>;
}

macro_rules! rows {
    ($t:ty,$method:ident) => {
        impl Rows for $t {
            fn rows(d: &Dataset, s: u64, n: u64) -> Result<Vec<Self>> {
                d.$method(s, n).map_err(h5)
            }
        }
    };
}

rows!(f32, read_f32_rows);

rows!(f64, read_f64_rows);

rows!(i8, read_i8_rows);

rows!(i16, read_i16_rows);

rows!(i32, read_i32_rows);

rows!(i64, read_i64_rows);

rows!(u8, read_u8_rows);

rows!(u16, read_u16_rows);

rows!(u32, read_u32_rows);

rows!(u64, read_u64_rows);
