use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub version: u32,
    pub bits: HashMap<u8, BitDef>,
    pub masks: HashMap<String, MaskDef>,
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

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("schema version mismatch: expected 1, got {0}")]
    VersionMismatch(u32),
    #[error("duplicate bit index: {0}")]
    DuplicateBitIndex(u8),
    #[error("bit index out of range (0-63): {0}")]
    BitIndexOutOfRange(u8),
    #[error("mask '{0}' references unknown bit index: {1}")]
    UnknownBitInMask(String, u8),
    #[error("mask '{0}' has duplicate bit indices")]
    DuplicateBitInMask(String),
    #[error("mask '{0}' is empty")]
    EmptyMask(String),
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

        for (&idx, _) in &self.bits {
            if idx > 63 {
                return Err(SchemaError::BitIndexOutOfRange(idx));
            }
        }

        let mut seen = std::collections::HashSet::new();
        for (&idx, _) in &self.bits {
            if !seen.insert(idx) {
                return Err(SchemaError::DuplicateBitIndex(idx));
            }
        }

        for (mask_name, mask) in &self.masks {
            if mask.bits.is_empty() {
                return Err(SchemaError::EmptyMask(mask_name.clone()));
            }
            let mut mask_seen = std::collections::HashSet::new();
            for &bit_idx in &mask.bits {
                if !self.bits.contains_key(&bit_idx) {
                    return Err(SchemaError::UnknownBitInMask(mask_name.clone(), bit_idx));
                }
                if !mask_seen.insert(bit_idx) {
                    return Err(SchemaError::DuplicateBitInMask(mask_name.clone()));
                }
            }
        }

        Ok(())
    }

    pub fn bit_index(&self, name: &str) -> Result<u8, SchemaError> {
        for (&idx, def) in &self.bits {
            if def.name == name {
                return Ok(idx);
            }
        }
        Err(SchemaError::BitNotFound(name.to_string()))
    }

    pub fn mask_bits(&self, name: &str) -> Result<u64, SchemaError> {
        let mask = self.masks.get(name).ok_or(SchemaError::MaskNotFound(name.to_string()))?;
        let mut bits: u64 = 0;
        for &idx in &mask.bits {
            bits |= 1u64 << idx;
        }
        Ok(bits)
    }

    pub fn missing_labels(&self, mask_name: &str, current_bits: u64) -> Result<Vec<String>, SchemaError> {
        let mask = self.masks.get(mask_name).ok_or(SchemaError::MaskNotFound(mask_name.to_string()))?;
        let required = self.mask_bits(mask_name)?;
        let missing = required & !current_bits;
        let mut labels = Vec::new();
        for &idx in &mask.bits {
            if missing & (1u64 << idx) != 0 {
                labels.push(self.bits[&idx].name.clone());
            }
        }
        Ok(labels)
    }

    pub fn all_bit_names(&self) -> Vec<(u8, String)> {
        let mut v: Vec<_> = self.bits.iter().map(|(&k, v)| (k, v.name.clone())).collect();
        v.sort_by_key(|(k, _)| *k);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_validation() {
        let mut bits = HashMap::new();
        bits.insert(0, BitDef { name: "auth".into(), desc: "User authenticated".into() });
        bits.insert(1, BitDef { name: "perm".into(), desc: "Has permission".into() });

        let mut masks = HashMap::new();
        masks.insert("required".into(), MaskDef { bits: vec![0, 1], desc: "Required".into() });

        let schema = Schema { version: 1, bits, masks };
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_mask_bits() {
        let mut bits = HashMap::new();
        bits.insert(0, BitDef { name: "a".into(), desc: "".into() });
        bits.insert(3, BitDef { name: "b".into(), desc: "".into() });

        let mut masks = HashMap::new();
        masks.insert("m1".into(), MaskDef { bits: vec![0, 3], desc: "".into() });

        let schema = Schema { version: 1, bits, masks };
        assert_eq!(schema.mask_bits("m1").unwrap(), 0b1001);
    }

    #[test]
    fn test_missing_labels() {
        let mut bits = HashMap::new();
        bits.insert(0, BitDef { name: "auth".into(), desc: "".into() });
        bits.insert(1, BitDef { name: "perm".into(), desc: "".into() });

        let mut masks = HashMap::new();
        masks.insert("req".into(), MaskDef { bits: vec![0, 1], desc: "".into() });

        let schema = Schema { version: 1, bits, masks };
        // current has only auth (bit 0)
        let missing = schema.missing_labels("req", 0b01).unwrap();
        assert_eq!(missing, vec!["perm"]);
    }
}