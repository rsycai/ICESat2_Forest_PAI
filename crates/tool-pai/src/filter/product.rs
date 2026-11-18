use super::Filter;
use crate::{model::BEAMS, output};
use anyhow::{Context, Result, bail, ensure};
use hdf5_pure::{DType, File, FileBuilder, LibVer};
use std::path::Path;

impl Filter<'_> {
    pub(crate) fn product(&self, source: &File, path: &Path) -> Result<()> {
        let mut builder = FileBuilder::new();
        builder.with_libver_bounds(LibVer::Earliest, LibVer::V18);
        let names = source.group("/")?.groups()?;
        ensure!(
            BEAMS.iter().any(|b| names.iter().any(|n| n == b)),
            "Intermediate product lacks beam groups"
        );
        for beam in BEAMS.into_iter().filter(|b| names.iter().any(|n| n == b)) {
            let group = source.group(beam)?;
            let fields = group.datasets()?;
            let dataset = |name: &str| source.dataset(&format!("{beam}/{name}"));
            let qa = dataset("QA")?.read::<i8>()?;
            let lon = dataset("longitude")?.read::<f32>()?;
            let lat = dataset("latitude")?.read::<f32>()?;
            let pai = dataset("PAI")?.read::<f32>()?;
            let status = fields
                .iter()
                .any(|s| s == "pai_status")
                .then(|| dataset("pai_status")?.read::<u8>())
                .transpose()?;
            let n = qa.len();
            ensure!(
                [lon.len(), lat.len(), pai.len()]
                    .into_iter()
                    .all(|v| v == n)
                    && status.as_ref().is_none_or(|s| s.len() == n),
                "{beam}: segment columns have inconsistent lengths"
            );
            let keep = self.indices((0..n).map(|i| {
                (
                    qa[i],
                    pai[i].is_finite() && pai[i] > 0.0 && status.as_ref().is_none_or(|s| s[i] == 0),
                    lon[i],
                    lat[i],
                )
            }))?;
            let mut out = builder.create_group(beam);
            for field in fields {
                if matches!(
                    field.as_str(),
                    "n_pulses" | "n_ground_pulses" | "gap_f_pulse" | "PAI_pulse" | "PAIe_pulse"
                ) {
                    continue;
                }
                let ds = dataset(&field)?;
                ensure!(ds.shape()? == [n as u64], "{beam}/{field} is not a consistent per-segment column");
                let target = out.create_dataset(&field);
                macro_rules! copy {
                    ($ty:ty) => {{
                        let data = ds
                            .read::<$ty>()
                            .with_context(|| format!("Read {beam}/{field}"))?;
                        target.with_data(&keep.iter().map(|&i| data[i]).collect::<Vec<_>>());
                    }};
                }
                match ds.dtype()? {
                    DType::F32 => copy!(f32),
                    DType::F64 => copy!(f64),
                    DType::I8 => copy!(i8),
                    DType::U8 => copy!(u8),
                    DType::I16 => copy!(i16),
                    DType::U16 => copy!(u16),
                    DType::I32 => copy!(i32),
                    DType::U32 => copy!(u32),
                    DType::I64 => copy!(i64),
                    DType::U64 => copy!(u64),
                    other => bail!("{beam}/{field}: unsupported data type for segment filtering: {other}"),
                }
                for (name, value) in ds.attrs()? {
                    target.set_attr(&name, value);
                }
            }
            builder.add_group(out.finish());
        }
        output::publish(path, builder)
    }
}
