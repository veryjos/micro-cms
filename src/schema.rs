use std::collections::HashMap;
use std::str::FromStr;

use std::error::Error;

use serde::Deserialize;

use crate::entity::{FieldType, FieldData};
use crate::parse::{keyval_map, FromKeyAndVal};

#[derive(Deserialize)]
pub struct FieldDeclaration {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub mutable: bool,
    #[serde(rename = "type")]
    pub ty: FieldType,
}

impl FromKeyAndVal for FieldDeclaration {
    fn from_key_and_val(key: &str, val: &str) -> Result<Self, Box<dyn Error>> {
        Ok(FieldDeclaration {
            name: key.to_owned(),
            ty: serde_plain::from_str(val)?,

            required: false,
            mutable: false,
        })
    }

    fn set_key(&mut self, key: &str) {
        self.name = key.to_owned();
    }
}

#[derive(Deserialize)]
pub struct EntityDeclaration {
    #[serde(deserialize_with = "keyval_map")]
    pub fields: HashMap<String, FieldDeclaration>,
}

pub trait FromFieldData {
    fn from_field_data<'a>(ty: &'a FieldType, data: &'a FieldData) -> Option<&'a Self>;
}

impl FromFieldData for String {
    fn from_field_data<'a>(ty: &'a FieldType, data: &'a FieldData) -> Option<&'a Self> {
        match (ty, data) {
            (FieldType::Str, FieldData::Str(s)) =>
                Some(&s),

            _ => None
        }
    }
}

impl FromFieldData for f64 {
    fn from_field_data<'a>(ty: &'a FieldType, data: &'a FieldData) -> Option<&'a Self> {
        match (ty, data) {
            (FieldType::Num, FieldData::Num(n)) =>
                Some(&n),

            _ => None
        }
    }
}

impl FromStr for EntityDeclaration {
    type Err = toml::de::Error;

    fn from_str(input: &str) -> Result<EntityDeclaration, toml::de::Error> {
        toml::from_str(input)
    }
}
