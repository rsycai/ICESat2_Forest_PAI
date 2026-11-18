use crate::output::hdf5::result::Result;
use hdf5_pure::AttrValue;
use hdf5_reader::Datatype;
use hdf5_writer::AttributeBuilder;

pub(super) fn contains_reference(datatype: &Datatype) -> bool {
    match datatype {
        Datatype::Reference { .. } => true,
        Datatype::Compound { fields, .. } => fields.iter().any(|f| contains_reference(&f.datatype)),
        Datatype::Array { base, .. } | Datatype::VarLen { base, .. } => contains_reference(base),
        _ => false,
    }
}

pub(super) fn attribute(name: &str, value: AttrValue) -> Result<AttributeBuilder> {
    macro_rules! values {
        ($($scalar:ident, $array:ident),*) => {
            match value {
                $(AttrValue::$scalar(v) => AttributeBuilder::scalar(name, v)?,
                  AttrValue::$array(v) => AttributeBuilder::vector(name, &v)?,)*
                AttrValue::String(v) | AttrValue::AsciiString(v)
                | AttrValue::VarLenString(v) | AttrValue::VarLenAsciiString(v)
                | AttrValue::StringSized { value: v, .. }
                | AttrValue::AsciiStringSized { value: v, .. } => AttributeBuilder::fixed_string(name, v),
                other => return Err(format!("Unsupported HDF5 attribute for writing: {name}: {other:?}").into()),
            }
        }
    }
    Ok(values!(
        F32, F32Array, F64, F64Array, I8, I8Array, I16, I16Array, I32, I32Array, I64, I64Array, U8,
        U8Array, U16, U16Array, U32, U32Array, U64, U64Array
    ))
}
