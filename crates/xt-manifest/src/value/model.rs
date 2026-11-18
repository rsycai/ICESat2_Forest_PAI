use crate::error::invalid;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use xt_error::Result;

#[derive(Clone, Debug, PartialEq)]
pub enum XmlValue {
    Text(String),
    Fields(Vec<(String, XmlValue)>),
}

impl XmlValue {
    pub fn encode(value: &impl Serialize) -> Result<Self> {
        let xml = quick_xml::se::to_string_with_root("details", value)
            .map_err(|e| invalid(format!("Failed to generate extended fields: {e}")))?;
        quick_xml::de::from_str(&xml).map_err(|e| invalid(format!("Failed to read extended fields: {e}")))
    }

    pub fn decode<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let xml = quick_xml::se::to_string_with_root("details", self)
            .map_err(|e| invalid(format!("Failed to generate extended fields: {e}")))?;
        quick_xml::de::from_str(&xml).map_err(|e| invalid(format!("Failed to read extended fields: {e}")))
    }
}

impl Serialize for XmlValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Text(text) => serializer.serialize_str(text),
            Self::Fields(fields) => {
                let mut map = serializer.serialize_map(Some(fields.len()))?;
                for (name, value) in fields {
                    map.serialize_entry(name, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for XmlValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct XmlVisitor;
        impl<'de> Visitor<'de> for XmlVisitor {
            type Value = XmlValue;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("XML text or field collection")
            }
            fn visit_str<E: serde::de::Error>(
                self,
                value: &str,
            ) -> std::result::Result<Self::Value, E> {
                Ok(XmlValue::Text(value.into()))
            }
            fn visit_string<E: serde::de::Error>(
                self,
                value: String,
            ) -> std::result::Result<Self::Value, E> {
                Ok(XmlValue::Text(value))
            }
            fn visit_map<M: MapAccess<'de>>(
                self,
                mut access: M,
            ) -> std::result::Result<Self::Value, M::Error> {
                let mut fields = Vec::new();
                while let Some(entry) = access.next_entry()? {
                    fields.push(entry);
                }
                Ok(XmlValue::Fields(fields))
            }
        }
        deserializer.deserialize_any(XmlVisitor)
    }
}
