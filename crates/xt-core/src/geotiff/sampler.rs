use super::{projection::Projection, raw::RawBlocks};
use geotiff_reader::GeoTiffFile;
use std::{collections::BTreeMap, fmt::Display, path::Path, str::FromStr};
use tiff_reader::{RasterLayout, TiffSample};
use xt_error::{DataError, Error, ErrorKind, GeoError, Result};

pub struct GeoTiffSampler {
    file: GeoTiffFile,
    projection: Projection,
    layout: RasterLayout,
    block: (usize, usize),
    raw: Option<RawBlocks>,
}

impl GeoTiffSampler {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file =
            GeoTiffFile::open(path).map_err(|e| read_error(format!("{}: {e}", path.display())))?;
        let crs = file.crs();
        let horizontal = crs.horizontal.as_ref().and_then(|h| match crs.model_type {
            1 => h.projected_epsg,
            2 => h.geodetic_epsg,
            0 => h.projected_epsg.or(h.geodetic_epsg),
            _ => None,
        });
        let projection = Projection::new(
            horizontal
                .filter(|id| !matches!(id, 0 | 32767))
                .map(u32::from),
        )?;
        if file.geo_to_pixel(0.0, 0.0).is_none() {
            return Err(Error::new(
                ErrorKind::Geo(GeoError::CoordinateTransform),
                "Missing invertible GeoTIFF affine transform",
            ));
        }
        let ifd = file.tiff().ifd(file.base_ifd_index()).map_err(read_error)?;
        let layout = ifd.raster_layout().map_err(read_error)?;
        let block = if ifd.is_tiled() {
            (
                ifd.tile_width().unwrap_or(0) as usize,
                ifd.tile_height().unwrap_or(0) as usize,
            )
        } else {
            (
                layout.width,
                (ifd.rows_per_strip() as usize).min(layout.height),
            )
        };
        if block.0 == 0 || block.1 == 0 {
            return Err(read_error("GeoTIFF tile dimensions must be nonzero"));
        }
        let raw = RawBlocks::new(ifd, &layout)?;
        Ok(Self {
            file,
            projection,
            layout,
            block,
            raw,
        })
    }

    pub fn sample<T>(&self, lon: f64, lat: f64) -> Result<Option<T>>
    where
        T: TiffSample + Copy + PartialOrd + FromStr,
    {
        Ok(self.sample_many(&[(lon, lat)])?[0])
    }

    pub fn sample_many<T>(&self, points: &[(f64, f64)]) -> Result<Vec<Option<T>>>
    where
        T: TiffSample + Copy + PartialOrd + FromStr,
    {
        self.sample_band_many(1, points)
    }

    pub fn sample_band_many<T>(&self, band: usize, points: &[(f64, f64)]) -> Result<Vec<Option<T>>>
    where
        T: TiffSample + Copy + PartialOrd + FromStr,
    {
        if !(1..=self.layout.samples_per_pixel).contains(&band) {
            return Err(Error::new(
                ErrorKind::Geo(GeoError::InvalidBand),
                format!("Band {band} is outside 1..={}", self.layout.samples_per_pixel),
            ));
        }
        if !T::matches_layout(&self.layout) {
            return Err(Error::new(
                ErrorKind::Data(DataError::SchemaMismatch),
                format!(
                    "Requested {} does not match the band type (SampleFormat={}, BitsPerSample={})",
                    T::type_name(),
                    self.layout.sample_format,
                    self.layout.bits_per_sample
                ),
            ));
        }
        let nodata = self.file.nodata().and_then(|text| {
            let text = text.trim();
            text.parse::<T>().ok().or_else(|| {
                let (integer, fraction) = text.split_once('.')?;
                fraction
                    .bytes()
                    .all(|b| b == b'0')
                    .then(|| integer.parse().ok())
                    .flatten()
            })
        });
        let valid = |v: &T| Some(*v) != nodata && v.partial_cmp(v).is_some();
        let mut values = vec![None; points.len()];
        let mut groups = BTreeMap::<_, Vec<_>>::new();
        for (index, &(lon, lat)) in points.iter().enumerate() {
            let (x, y) = self.projection.coordinates(lon, lat)?;
            let (col, row) = self.file.geo_to_pixel(x, y).expect("Validated affine transform");
            if !(0.0..self.layout.width as f64).contains(&col)
                || !(0.0..self.layout.height as f64).contains(&row)
            {
                continue;
            }
            let (row, col) = (row as usize, col as usize);
            if let Some(raw) = &self.raw {
                values[index] = raw
                    .sample(&self.file, &self.layout, self.block, (row, col), band - 1)?
                    .filter(valid);
            } else {
                groups
                    .entry((row / self.block.1, col / self.block.0))
                    .or_default()
                    .push((index, row, col));
            }
        }
        for group in groups.values() {
            let (_, first_row, first_col) = group[0];
            let (mut top, mut bottom, mut left, mut right) =
                (first_row, first_row, first_col, first_col);
            for &(_, row, col) in group {
                top = top.min(row);
                bottom = bottom.max(row);
                left = left.min(col);
                right = right.max(col);
            }
            let window = self
                .file
                .read_band_window::<T>(band - 1, top, left, bottom - top + 1, right - left + 1)
                .map_err(read_error)?;
            for &(index, row, col) in group {
                values[index] = Some(window[[row - top, col - left]]).filter(valid);
            }
        }
        Ok(values)
    }
}

pub(super) fn read_error(error: impl Display) -> Error {
    Error::new(
        ErrorKind::Data(DataError::Parse),
        format!("Failed to read GeoTIFF: {error}"),
    )
}
