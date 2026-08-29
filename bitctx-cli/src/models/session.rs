use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub schema_hash: String,
    pub bits: u64,
    pub created_at: String,
    pub updated_at: String,
}

impl Session {
    pub fn new(id: String, schema_hash: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            schema_hash,
            bits: 0,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn set_bit(&mut self, index: u8, value: bool) {
        if value {
            self.bits |= 1u64 << index;
        } else {
            self.bits &= !(1u64 << index);
        }
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn set_bits(&mut self, indices: &[u8], values: &[bool]) {
        for (&idx, &val) in indices.iter().zip(values.iter()) {
            self.set_bit(idx, val);
        }
    }

    pub fn get_bit(&self, index: u8) -> bool {
        (self.bits >> index) & 1 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_bit_ops() {
        let mut s = Session::new("test".into(), "hash".into());
        assert_eq!(s.bits, 0);

        s.set_bit(0, true);
        assert_eq!(s.bits, 1);
        assert!(s.get_bit(0));
        assert!(!s.get_bit(1));

        s.set_bit(3, true);
        assert_eq!(s.bits, 0b1001);

        s.set_bit(0, false);
        assert_eq!(s.bits, 0b1000);
        assert!(!s.get_bit(0));
    }
}
