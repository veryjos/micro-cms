use std::fmt;
use std::collections::HashMap;

use std::str::FromStr;

use serde::de;
use serde::{Serialize, Deserialize, Deserializer};

#[derive(Debug)]
pub enum FieldType {
    Str,
    Bin,
    Num,

    Ref(String)
}

#[derive(Debug, Serialize)]
pub enum FieldData {
    Str(String),
    Bin(Vec<u8>),
    Num(f64)
}

impl<'de> Deserialize<'de> for FieldData {
    fn deserialize<D>(deser: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        struct FieldDataVisitor;

        impl<'de> de::Visitor<'de> for FieldDataVisitor {
            type Value = FieldData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "field value")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                Ok(FieldData::Str(v.to_owned()))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                Ok(FieldData::Num(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                self.visit_f64(v as f64)
            }
        }

        deser.deserialize_any(FieldDataVisitor {})
    }
}

impl<'de> Deserialize<'de> for FieldType {
    fn deserialize<D>(deser: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        struct FieldTypeVisitor;

        impl<'de> de::Visitor<'de> for FieldTypeVisitor {
            type Value = FieldType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "field type")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                Ok(match s {
                    "string" | "str" => FieldType::Str,
                    "binary" | "bin" => FieldType::Bin,
                    "number" | "num" => FieldType::Num,

                    _ => FieldType::Ref(s.to_owned())
                })
            }
        }

        deser.deserialize_str(FieldTypeVisitor {})
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    #[serde(flatten)]
    pub fields: HashMap<String, FieldData>,
}

impl FromStr for Entity {
    type Err = toml::de::Error;

    fn from_str(input: &str) -> Result<Entity, toml::de::Error> {
        toml::from_str(input)
    }
}
