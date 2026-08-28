use crate::models::{Schema, Session};
use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

const SCHEMA_FILE: &str = "schema.json";

pub fn schema_path(session_id: &str) -> std::path::PathBuf {
    crate::models::ensure_bitctx_dir().unwrap().join(session_id).join(SCHEMA_FILE)
}

pub fn session_dir(session_id: &str) -> std::path::PathBuf {
    crate::models::ensure_bitctx_dir().unwrap().join(session_id)
}

fn session_file(session_id: &str) -> std::path::PathBuf {
    session_dir(session_id).join("session.json")
}

pub fn load_schema(session_id: &str) -> Result<Schema> {
    let path = schema_path(session_id);
    let file = File::open(&path).with_context(|| format!("schema not found: {}", path.display()))?;
    let reader = BufReader::new(file);
    let schema: Schema = serde_json::from_reader(reader).context("failed to parse schema")?;
    schema.validate().context("schema validation failed")?;
    Ok(schema)
}

pub fn save_schema(session_id: &str, schema: &Schema) -> Result<()> {
    let dir = session_dir(session_id);
    fs::create_dir_all(&dir).context("failed to create session dir")?;

    let path = schema_path(session_id);
    let tmp_path = path.with_extension("json.tmp");

    {
        let file = File::create(&tmp_path).context("failed to create temp schema file")?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, schema).context("failed to serialize schema")?;
        writer.flush().context("failed to flush schema")?;
    }

    fs::rename(&tmp_path, &path).context("failed to rename schema file")?;
    Ok(())
}

pub fn load_session(session_id: &str, expected_schema_hash: &str) -> Result<Session> {
    let path = session_file(session_id);
    let file = File::open(&path).with_context(|| format!("session not found: {}", path.display()))?;
    file.lock_shared().context("failed to lock session file")?;

    let mut contents = String::new();
    BufReader::new(&file).read_to_string(&mut contents).context("failed to read session")?;

    file.unlock().context("failed to unlock session file")?;

    let session: Session = serde_json::from_str(&contents).context("failed to parse session")?;

    if session.schema_hash != expected_schema_hash {
        anyhow::bail!(
            "schema hash mismatch: session has '{}', expected '{}'",
            session.schema_hash,
            expected_schema_hash
        );
    }

    Ok(session)
}

pub fn load_or_create_session(session_id: &str, expected_schema_hash: &str) -> Result<Session> {
    let path = session_file(session_id);
    if !path.exists() {
        return Ok(Session::new(session_id.to_string(), expected_schema_hash.to_string()));
    }
    load_session(session_id, expected_schema_hash)
}

pub fn save_session(session: &Session) -> Result<()> {
    let path = session_file(&session.id);
    let tmp_path = path.with_extension("json.tmp");

    let file = File::create(&tmp_path).context("failed to create temp session file")?;
    file.lock_exclusive().context("failed to lock session file for write")?;

    {
        let mut writer = BufWriter::new(&file);
        serde_json::to_writer_pretty(&mut writer, session).context("failed to serialize session")?;
        writer.flush().context("failed to flush session")?;
    }

    file.unlock().context("failed to unlock session file")?;
    drop(file);

    fs::rename(&tmp_path, &path).context("failed to rename session file")?;
    Ok(())
}

pub fn delete_session(session_id: &str) -> Result<()> {
    let dir = session_dir(session_id);
    if dir.exists() {
        fs::remove_dir_all(&dir).context("failed to delete session directory")?;
    }
    Ok(())
}

pub fn schema_hash(schema: &Schema) -> String {
    use twox_hash::XxHash64;
    use std::hash::{Hash, Hasher};

    let mut hasher = XxHash64::default();
    // Use canonical JSON with sorted keys for stable hashing
    let canonical = canonical_schema_json(schema);
    canonical.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn canonical_schema_json(schema: &Schema) -> String {
    use serde_json::{Map, Value};
    
    let mut bits_map = Map::new();
    let mut sorted_bits: Vec<_> = schema.bits.iter().collect();
    sorted_bits.sort_by_key(|(k, _)| *k);
    for (k, v) in sorted_bits {
        let mut bit_obj = Map::new();
        bit_obj.insert("name".into(), Value::String(v.name.clone()));
        bit_obj.insert("desc".into(), Value::String(v.desc.clone()));
        bits_map.insert(k.to_string(), Value::Object(bit_obj));
    }
    
    let mut masks_map = Map::new();
    let mut sorted_masks: Vec<_> = schema.masks.iter().collect();
    sorted_masks.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in sorted_masks {
        let mut mask_obj = Map::new();
        mask_obj.insert("bits".into(), Value::Array(v.bits.iter().map(|b| Value::Number((*b).into())).collect()));
        mask_obj.insert("desc".into(), Value::String(v.desc.clone()));
        masks_map.insert(k.clone(), Value::Object(mask_obj));
    }
    
    let mut root = Map::new();
    root.insert("version".into(), Value::Number(schema.version.into()));
    root.insert("bits".into(), Value::Object(bits_map));
    root.insert("masks".into(), Value::Object(masks_map));
    
    serde_json::to_string(&Value::Object(root)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BitDef, MaskDef, Schema};
    use std::collections::HashMap;

    fn test_schema() -> Schema {
        let mut bits = HashMap::new();
        bits.insert(0, BitDef { name: "auth".into(), desc: "".into() });
        bits.insert(1, BitDef { name: "perm".into(), desc: "".into() });

        let mut masks = HashMap::new();
        masks.insert("req".into(), MaskDef { bits: vec![0, 1], desc: "".into() });

        Schema { version: 1, bits, masks }
    }

    #[test]
    fn test_schema_roundtrip() {
        let schema = test_schema();
        let session_id = "test-roundtrip";

        save_schema(session_id, &schema).unwrap();
        let loaded = load_schema(session_id).unwrap();

        assert_eq!(loaded.version, schema.version);
        assert_eq!(loaded.bits.len(), schema.bits.len());
        assert_eq!(loaded.masks.len(), schema.masks.len());

        delete_session(session_id).unwrap();
    }

    #[test]
    fn test_session_roundtrip() {
        let schema = test_schema();
        let hash = schema_hash(&schema);
        let session_id = "test-session-rt";

        save_schema(session_id, &schema).unwrap();

        let mut session = Session::new(session_id.into(), hash.clone());
        session.set_bit(0, true);
        save_session(&session).unwrap();

        let loaded = load_session(session_id, &hash).unwrap();
        assert_eq!(loaded.bits, session.bits);
        assert!(loaded.get_bit(0));
        assert!(!loaded.get_bit(1));

        delete_session(session_id).unwrap();
    }

    #[test]
    fn test_schema_hash_mismatch() {
        let schema = test_schema();
        let hash = schema_hash(&schema);
        let session_id = "test-hash-mismatch";

        save_schema(session_id, &schema).unwrap();

        let mut session = Session::new(session_id.into(), "wrong-hash".into());
        save_session(&session).unwrap();

        let result = load_session(session_id, &hash);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("schema hash mismatch"));

        delete_session(session_id).unwrap();
    }
}