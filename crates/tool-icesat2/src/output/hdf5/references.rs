use crate::output::hdf5::attribute::contains_reference;
use crate::output::hdf5::original::Original;
use crate::output::hdf5::result::Result;
use hdf5_reader::{Attribute, Datatype};
use hdf5_writer::AttributeBuilder;
use std::collections::BTreeSet;

pub(super) struct Context {
    pub(super) originals: [Original; 2],
    pub(super) retained: BTreeSet<String>,
}

impl Context {
    pub(super) fn original(&self, path: &str) -> Option<(&Original, String)> {
        self.originals.iter().find_map(|s| {
            if path == s.product {
                Some((s, "/".into()))
            } else {
                path.strip_prefix(&format!("{}/", s.product))
                    .map(|p| (s, format!("/{p}")))
            }
        })
    }
    pub(super) fn attributes(
        &self,
        path: &str,
        is_dataset: bool,
    ) -> Result<Option<Vec<AttributeBuilder>>> {
        let Some((source, old_path)) = self.original(path) else {
            return Ok(None);
        };
        let values = if is_dataset {
            source.file.dataset(&old_path)?.attributes()
        } else {
            source.file.group(&old_path)?.attributes()?
        };
        let product = source.product;
        let mut result = Vec::new();
        for value in values {
            let converted = if value.name == "DIMENSION_LIST" {
                let root = source.file.root_group()?;
                let width = 8 + root.offset_size() as usize;
                let mut mapped = Vec::new();
                for reference in value.raw_data.chunks_exact(width) {
                    let bytes =
                        root.resolve_vlen_reference_bytes(reference, root.offset_size() as usize)?;
                    let mut sequence = Vec::new();
                    for raw in bytes.chunks_exact(root.offset_size() as usize) {
                        let old = source.target(raw)?;
                        let new = format!("{product}{old}");
                        if !self.retained.contains(&new) {
                            return Err(format!("Dimension scale {new} was not retained; refusing an invalid reference").into());
                        }
                        sequence.push(new);
                    }
                    mapped.push(sequence);
                }
                AttributeBuilder::vlen_object_references(&value.name, mapped)
            } else if value.name == "REFERENCE_LIST" {
                let Datatype::Compound { size, fields } = &value.datatype else {
                    return Err("REFERENCE_LIST must have a compound type".into());
                };
                let reference = fields
                    .iter()
                    .find(|f| f.name == "dataset")
                    .ok_or("REFERENCE_LIST is missing dataset")?;
                let dimension = fields
                    .iter()
                    .find(|f| f.name == "dimension")
                    .ok_or("REFERENCE_LIST is missing dimension")?;
                let mut mapped = Vec::new();
                let width = source.file.root_group()?.offset_size() as usize;
                for entry in value.raw_data.chunks_exact(*size as usize) {
                    let start = reference.byte_offset as usize;
                    let old = source.target(&entry[start..start + width])?;
                    let start = dimension.byte_offset as usize;
                    let number = Attribute {
                        name: String::new(),
                        datatype: dimension.datatype.clone(),
                        shape: vec![],
                        raw_data: entry[start..start + 4].to_vec(),
                        decoded_strings: None,
                    }
                    .read_scalar::<u32>()?;
                    let new = format!("{product}{old}");
                    if self.retained.contains(&new) {
                        mapped.push((new, number));
                    }
                }
                AttributeBuilder::object_reference_list(&value.name, mapped)
            } else if contains_reference(&value.datatype) {
                return Err(format!(
                    "Cannot relocate reference attribute {product}{old_path}/@{}; refusing to copy an invalid address",
                    value.name
                )
                .into());
            } else if matches!(
                &value.datatype,
                Datatype::VarLen { .. }
                    | Datatype::String {
                        size: hdf5_reader::StringSize::Variable,
                        ..
                    }
            ) {
                let strings = value.read_strings()?;
                let width = strings.iter().map(|s| s.len() + 1).max().unwrap_or(1);
                let mut bytes = Vec::new();
                for text in strings {
                    let mut slot = text.into_bytes();
                    slot.resize(width, 0);
                    bytes.extend(slot);
                }
                AttributeBuilder::new(
                    &value.name,
                    Datatype::String {
                        size: hdf5_reader::StringSize::Fixed(width as u32),
                        encoding: hdf5_reader::StringEncoding::Utf8,
                        padding: hdf5_reader::StringPadding::NullTerminate,
                    },
                    value.shape.clone(),
                    bytes,
                )
            } else {
                AttributeBuilder::new(
                    &value.name,
                    value.datatype.clone(),
                    value.shape.clone(),
                    value.raw_data.clone(),
                )
            };
            result.push(converted);
        }
        Ok(Some(result))
    }
}
