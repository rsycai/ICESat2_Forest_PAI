use geotiff_reader::GeoTiffFile;
use tiff_reader::{ByteOrder, Ifd, RasterLayout, TiffSample};
use xt_error::{DataError, Error, ErrorKind, Result};

pub(super) struct RawBlocks {
    offsets: Vec<u64>,
    sizes: Vec<u64>,
}

impl RawBlocks {
    pub fn new(ifd: &Ifd, layout: &RasterLayout) -> Result<Option<Self>> {
        if ifd.compression() != 1
            || layout.predictor != 1
            || ifd.fill_order() != 1
            || !matches!(layout.bits_per_sample, 8 | 16 | 32 | 64)
            || !matches!(ifd.photometric_interpretation().unwrap_or(1), 0..=3)
        {
            return Ok(None);
        }
        let (offsets, sizes) = if ifd.is_tiled() {
            (ifd.tile_offsets(), ifd.tile_byte_counts())
        } else {
            (ifd.strip_offsets(), ifd.strip_byte_counts())
        };
        let (offsets, sizes) = offsets.zip(sizes).ok_or_else(invalid_offset)?;
        Ok(Some(Self { offsets, sizes }))
    }

    pub fn sample<T: TiffSample>(
        &self,
        file: &GeoTiffFile,
        layout: &RasterLayout,
        block: (usize, usize),
        pixel: (usize, usize),
        band: usize,
    ) -> Result<Option<T>> {
        let (width, height) = block;
        let (row, col) = pixel;
        let across = layout.width.div_ceil(width);
        let mut index = row / height * across + col / width;
        let within = (row % height) as u64 * width as u64 + (col % width) as u64;
        let sample = if layout.planar_configuration == 2 {
            let plane = across
                .checked_mul(layout.height.div_ceil(height))
                .ok_or_else(invalid_offset)?;
            index = band
                .checked_mul(plane)
                .and_then(|v| v.checked_add(index))
                .ok_or_else(invalid_offset)?;
            within
        } else {
            within
                .checked_mul(layout.samples_per_pixel as u64)
                .and_then(|v| v.checked_add(band as u64))
                .ok_or_else(invalid_offset)?
        };
        let (&base, &size) = self
            .offsets
            .get(index)
            .zip(self.sizes.get(index))
            .ok_or_else(invalid_offset)?;
        if base == 0 && size == 0 {
            return Ok(None);
        }
        let bytes = layout.bytes_per_sample;
        let offset = sample
            .checked_mul(bytes as u64)
            .ok_or_else(invalid_offset)?;
        if base == 0
            || offset
                .checked_add(bytes as u64)
                .is_none_or(|end| end > size)
        {
            return Err(invalid_offset());
        }
        let mut value = file
            .tiff()
            .source()
            .read_exact_at(base.checked_add(offset).ok_or_else(invalid_offset)?, bytes)
            .map_err(super::sampler::read_error)?;
        if matches!(file.tiff().byte_order(), ByteOrder::LittleEndian)
            != cfg!(target_endian = "little")
        {
            value.reverse();
        }
        T::decode_many(&value)
            .pop()
            .map(Some)
            .ok_or_else(invalid_offset)
    }
}

fn invalid_offset() -> Error {
    Error::new(
        ErrorKind::Data(DataError::Corrupt),
        "Invalid GeoTIFF tile offset or byte count",
    )
}
