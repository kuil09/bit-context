use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_mask: Option<String>,
    #[serde(deserialize_with = "deserialize_bits")]
    pub bits: BTreeMap<u8, BitDef>,
    #[serde(deserialize_with = "deserialize_masks")]
    pub masks: BTreeMap<String, MaskDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitDef {
    pub name: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskDef {
    pub bits: Vec<u8>,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissingCondition {
    pub index: u8,
    pub name: String,
    pub desc: String,
}

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("schema version mismatch: expected 1, got {0}")]
    VersionMismatch(u32),
    #[error("bit index out of range (0-63): {0}")]
    BitIndexOutOfRange(u8),
    #[error("bit at index {0} has an empty name")]
    EmptyBitName(u8),
    #[error("bit name '{0}' has leading or trailing whitespace")]
    BitNameWhitespace(String),
    #[error("bit name '{0}' contains a comma or control character")]
    InvalidBitName(String),
    #[error("duplicate bit name: '{0}'")]
    DuplicateBitName(String),
    #[error("mask name must not be empty")]
    EmptyMaskName,
    #[error("mask name '{0}' contains a control character")]
    InvalidMaskName(String),
    #[error("mask '{0}' references unknown bit index: {1}")]
    UnknownBitInMask(String, u8),
    #[error("mask '{0}' has duplicate bit index: {1}")]
    DuplicateBitInMask(String, u8),
    #[error("mask '{0}' is empty")]
    EmptyMask(String),
    #[error("default mask '{0}' not found in schema")]
    DefaultMaskNotFound(String),
    #[error("bit '{0}' not found in schema")]
    BitNotFound(String),
    #[error("mask '{0}' not found in schema")]
    MaskNotFound(String),
}

impl Schema {
    pub fn validate(&self) -> Result<(), SchemaError> {
        if self.version != 1 {
            return Err(SchemaError::VersionMismatch(self.version));
        }

        let mut bit_names = BTreeSet::new();
        for (&index, bit) in &self.bits {
            if index > 63 {
                return Err(SchemaError::BitIndexOutOfRange(index));
            }
            if bit.name.is_empty() {
                return Err(SchemaError::EmptyBitName(index));
            }
            if bit.name.trim() != bit.name {
                return Err(SchemaError::BitNameWhitespace(bit.name.clone()));
            }
            if bit.name.contains(',') || bit.name.chars().any(char::is_control) {
                return Err(SchemaError::InvalidBitName(bit.name.clone()));
            }
            if !bit_names.insert(bit.name.clone()) {
                return Err(SchemaError::DuplicateBitName(bit.name.clone()));
            }
        }

        for (mask_name, mask) in &self.masks {
            if mask_name.trim().is_empty() {
                return Err(SchemaError::EmptyMaskName);
            }
            if mask_name.chars().any(char::is_control) {
                return Err(SchemaError::InvalidMaskName(mask_name.clone()));
            }
            if mask.bits.is_empty() {
                return Err(SchemaError::EmptyMask(mask_name.clone()));
            }

            let mut seen = BTreeSet::new();
            for &index in &mask.bits {
                if !self.bits.contains_key(&index) {
                    return Err(SchemaError::UnknownBitInMask(mask_name.clone(), index));
                }
                if !seen.insert(index) {
                    return Err(SchemaError::DuplicateBitInMask(mask_name.clone(), index));
                }
            }
        }

        if let Some(default_mask) = &self.default_mask {
            if !self.masks.contains_key(default_mask) {
                return Err(SchemaError::DefaultMaskNotFound(default_mask.clone()));
            }
        }

        Ok(())
    }

    pub fn bit_index(&self, name: &str) -> Result<u8, SchemaError> {
        self.bits
            .iter()
            .find_map(|(&index, bit)| (bit.name == name).then_some(index))
            .ok_or_else(|| SchemaError::BitNotFound(name.to_string()))
    }

    pub fn missing_conditions(
        &self,
        mask_name: &str,
        current_bits: u64,
    ) -> Result<Vec<MissingCondition>, SchemaError> {
        let mask = self
            .masks
            .get(mask_name)
            .ok_or_else(|| SchemaError::MaskNotFound(mask_name.to_string()))?;

        Ok(mask
            .bits
            .iter()
            .filter(|&&index| current_bits & (1_u64 << index) == 0)
            .map(|&index| {
                let bit = &self.bits[&index];
                MissingCondition {
                    index,
                    name: bit.name.clone(),
                    desc: bit.desc.clone(),
                }
            })
            .collect())
    }

    pub fn all_bit_names(&self) -> Vec<(u8, String)> {
        self.bits
            .iter()
            .map(|(&index, bit)| (index, bit.name.clone()))
            .collect()
    }
}

fn deserialize_bits<'de, D>(deserializer: D) -> Result<BTreeMap<u8, BitDef>, D::Error>
where
    D: Deserializer<'de>,
{
    struct BitsVisitor;

    impl<'de> Visitor<'de> for BitsVisitor {
        type Value = BTreeMap<u8, BitDef>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an object mapping bit indices to bit definitions")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut bits = BTreeMap::new();
            while let Some((raw_index, bit)) = map.next_entry::<String, BitDef>()? {
                let index = raw_index.parse::<u8>().map_err(|_| {
                    de::Error::custom(format!("invalid bit index key: '{raw_index}'"))
                })?;
                if bits.insert(index, bit).is_some() {
                    return Err(de::Error::custom(format!("duplicate bit index: {index}")));
                }
            }
            Ok(bits)
        }
    }

    deserializer.deserialize_map(BitsVisitor)
}

fn deserialize_masks<'de, D>(deserializer: D) -> Result<BTreeMap<String, MaskDef>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MasksVisitor;

    impl<'de> Visitor<'de> for MasksVisitor {
        type Value = BTreeMap<String, MaskDef>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an object mapping mask names to mask definitions")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut masks = BTreeMap::new();
            while let Some((name, mask)) = map.next_entry::<String, MaskDef>()? {
                if masks.insert(name.clone(), mask).is_some() {
                    return Err(de::Error::custom(format!("duplicate mask name: '{name}'")));
                }
            }
            Ok(masks)
        }
    }

    deserializer.deserialize_map(MasksVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_schema() -> Schema {
        Schema {
            version: 1,
            default_mask: Some("required".into()),
            bits: BTreeMap::from([
                (
                    0,
                    BitDef {
                        name: "auth".into(),
                        desc: "User authenticated".into(),
                    },
                ),
                (
                    3,
                    BitDef {
                        name: "승인".into(),
                        desc: "한국어 설명".into(),
                    },
                ),
            ]),
            masks: BTreeMap::from([(
                "required".into(),
                MaskDef {
                    bits: vec![3, 0],
                    desc: "Required".into(),
                },
            )]),
        }
    }

    #[test]
    fn validates_unicode_and_preserves_mask_order() {
        let schema = test_schema();
        schema.validate().expect("schema should be valid");

        let missing = schema
            .missing_conditions("required", 0)
            .expect("mask should exist");
        assert_eq!(missing[0].index, 3);
        assert_eq!(missing[0].name, "승인");
        assert_eq!(missing[1].index, 0);
        assert_eq!(missing[1].name, "auth");
    }

    #[test]
    fn rejects_duplicate_bit_names() {
        let mut schema = test_schema();
        schema.bits.get_mut(&3).expect("bit should exist").name = "auth".into();

        assert!(matches!(
            schema.validate(),
            Err(SchemaError::DuplicateBitName(name)) if name == "auth"
        ));
    }

    #[test]
    fn rejects_invalid_bit_names() {
        let mut schema = test_schema();
        schema.bits.get_mut(&0).expect("bit should exist").name = "auth,admin".into();

        assert!(matches!(
            schema.validate(),
            Err(SchemaError::InvalidBitName(name)) if name == "auth,admin"
        ));
    }

    #[test]
    fn rejects_duplicate_json_bit_indices() {
        let input = r#"{
            "version": 1,
            "bits": {
                "1": {"name": "a", "desc": ""},
                "01": {"name": "b", "desc": ""}
            },
            "masks": {"m": {"bits": [1], "desc": ""}}
        }"#;

        let error = serde_json::from_str::<Schema>(input).expect_err("duplicate index must fail");
        assert!(error.to_string().contains("duplicate bit index: 1"));
    }

    #[test]
    fn rejects_duplicate_json_mask_names() {
        let input = r#"{
            "version": 1,
            "bits": {"1": {"name": "a", "desc": ""}},
            "masks": {
                "m": {"bits": [1], "desc": "first"},
                "m": {"bits": [1], "desc": "second"}
            }
        }"#;

        let error = serde_json::from_str::<Schema>(input).expect_err("duplicate mask must fail");
        assert!(error.to_string().contains("duplicate mask name: 'm'"));
    }

    #[test]
    fn rejects_unknown_default_mask() {
        let mut schema = test_schema();
        schema.default_mask = Some("unknown".into());

        assert!(matches!(
            schema.validate(),
            Err(SchemaError::DefaultMaskNotFound(name)) if name == "unknown"
        ));
    }
}
