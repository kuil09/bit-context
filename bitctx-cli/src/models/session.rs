use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn eval_mask(&self, mask: u64) -> (bool, u64) {
        let missing = mask & !self.bits;
        let pass = missing == 0;
        (pass, missing)
    }
}

pub fn session_path(session_id: &str) -> PathBuf {
    let home = dirs::home_dir().expect("home directory not found");
    home.join(".bitctx").join(format!("{}.json", session_id))
}

pub fn ensure_bitctx_dir() -> std::io::Result<PathBuf> {
    let home = dirs::home_dir().expect("home directory not found");
    let dir = home.join(".bitctx");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
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

    #[test]
    fn test_eval_mask() {
        let mut s = Session::new("test".into(), "hash".into());
        s.set_bit(0, true);
        s.set_bit(2, true); // bits: 0b101

        let mask = 0b101; // need bits 0 and 2
        let (pass, missing) = s.eval_mask(mask);
        assert!(pass);
        assert_eq!(missing, 0);

        let mask2 = 0b111; // need bits 0, 1, 2
        let (pass2, missing2) = s.eval_mask(mask2);
        assert!(!pass2);
        assert_eq!(missing2, 0b010); // bit 1 missing
    }
}