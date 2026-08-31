use crate::models::{Schema, Session};
use anyhow::{Context, Result, anyhow, bail};
use fs2::FileExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

const SCHEMA_FILE: &str = "schema.json";
const SESSION_FILE: &str = "session.json";
const LOCKS_DIR: &str = ".locks";

#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn from_data_dir(data_dir: Option<PathBuf>) -> Result<Self> {
        let root = match data_dir {
            Some(path) => path,
            None => dirs::home_dir()
                .context("could not determine the home directory")?
                .join(".bitctx"),
        };

        Self::new(root)
    }

    pub fn new(root: PathBuf) -> Result<Self> {
        let root = if root.is_absolute() {
            root
        } else {
            std::env::current_dir()
                .context("could not resolve the current directory")?
                .join(root)
        };

        Ok(Self { root })
    }

    #[cfg(test)]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn initialize(&self, session_id: &str, schema: &Schema, force: bool) -> Result<Session> {
        validate_session_id(session_id)?;
        schema.validate().context("schema validation failed")?;

        let _lock = self.lock(session_id, LockMode::Exclusive)?;
        let session_dir = self.checked_session_dir(session_id)?;

        if session_dir.exists() {
            if !force {
                bail!("session '{session_id}' already exists; pass --force to reinitialize it");
            }
            self.remove_session_dir_checked(&session_dir)?;
        }

        create_private_dir(&session_dir).context("failed to create session directory")?;
        let session = Session::new(session_id.to_string(), schema_hash(schema));
        self.write_json_atomic(&session_dir.join(SCHEMA_FILE), schema)?;
        self.write_json_atomic(&session_dir.join(SESSION_FILE), &session)?;
        Ok(session)
    }

    pub fn read_session(&self, session_id: &str) -> Result<(Schema, Session)> {
        validate_session_id(session_id)?;
        let _lock = self.lock(session_id, LockMode::Shared)?;
        self.read_session_unlocked(session_id)
    }

    pub fn update_session<T, F>(&self, session_id: &str, update: F) -> Result<T>
    where
        F: FnOnce(&Schema, &mut Session) -> Result<T>,
    {
        validate_session_id(session_id)?;
        let _lock = self.lock(session_id, LockMode::Exclusive)?;
        let (schema, mut session) = self.read_session_unlocked(session_id)?;
        let result = update(&schema, &mut session)?;
        let path = self.checked_session_dir(session_id)?.join(SESSION_FILE);
        self.write_json_atomic(&path, &session)?;
        Ok(result)
    }

    pub fn reset(&self, session_id: &str) -> Result<bool> {
        validate_session_id(session_id)?;
        let _lock = self.lock(session_id, LockMode::Exclusive)?;
        let session_dir = self.checked_session_dir(session_id)?;

        if !session_dir.exists() {
            return Ok(false);
        }

        self.remove_session_dir_checked(&session_dir)?;
        Ok(true)
    }

    fn read_session_unlocked(&self, session_id: &str) -> Result<(Schema, Session)> {
        let session_dir = self.checked_session_dir(session_id)?;
        let metadata = fs::symlink_metadata(&session_dir).with_context(|| {
            format!(
                "session '{}' is not initialized in {}",
                session_id,
                self.root.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            bail!(
                "session path is not a regular directory: {}",
                session_dir.display()
            );
        }

        let schema_path = session_dir.join(SCHEMA_FILE);
        let schema: Schema = read_json(&schema_path)
            .with_context(|| format!("failed to read schema: {}", schema_path.display()))?;
        schema.validate().context("schema validation failed")?;

        let session_path = session_dir.join(SESSION_FILE);
        let session: Session = read_json(&session_path)
            .with_context(|| format!("failed to read session: {}", session_path.display()))?;

        if session.id != session_id {
            bail!(
                "session id mismatch: state has '{}', expected '{}'",
                session.id,
                session_id
            );
        }

        let expected_hash = schema_hash(&schema);
        if session.schema_hash != expected_hash {
            bail!(
                "schema hash mismatch: session has '{}', expected '{}'",
                session.schema_hash,
                expected_hash
            );
        }

        Ok((schema, session))
    }

    fn lock(&self, session_id: &str, mode: LockMode) -> Result<SessionLock> {
        self.ensure_root()?;
        let locks_dir = self.root.join(LOCKS_DIR);
        create_private_dir(&locks_dir).context("failed to create lock directory")?;

        let lock_path = locks_dir.join(format!("{session_id}.lock"));
        let file = open_private_file(&lock_path)?;
        match mode {
            LockMode::Shared => FileExt::lock_shared(&file),
            LockMode::Exclusive => FileExt::lock_exclusive(&file),
        }
        .with_context(|| {
            format!(
                "failed to lock session '{}': {}",
                session_id,
                lock_path.display()
            )
        })?;

        Ok(SessionLock { file })
    }

    fn ensure_root(&self) -> Result<()> {
        if let Ok(metadata) = fs::symlink_metadata(&self.root) {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!(
                    "data directory is not a regular directory: {}",
                    self.root.display()
                );
            }
        }
        create_private_dir(&self.root).context("failed to create data directory")
    }

    fn checked_session_dir(&self, session_id: &str) -> Result<PathBuf> {
        validate_session_id(session_id)?;
        let path = self.root.join(session_id);
        if path.parent() != Some(self.root.as_path()) {
            bail!("session path is not directly below the data directory");
        }
        Ok(path)
    }

    fn remove_session_dir_checked(&self, session_dir: &Path) -> Result<()> {
        if session_dir.parent() != Some(self.root.as_path()) {
            bail!("refusing to remove a path outside the data directory");
        }

        let metadata = fs::symlink_metadata(session_dir).with_context(|| {
            format!("failed to inspect session path: {}", session_dir.display())
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            bail!(
                "refusing to remove a non-directory session path: {}",
                session_dir.display()
            );
        }

        fs::remove_dir_all(session_dir).with_context(|| {
            format!(
                "failed to delete session directory: {}",
                session_dir.display()
            )
        })
    }

    fn write_json_atomic<T>(&self, path: &Path, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        let parent = path
            .parent()
            .ok_or_else(|| anyhow!("state file has no parent directory: {}", path.display()))?;
        if parent.parent() != Some(self.root.as_path()) {
            bail!(
                "refusing to write outside a session directory: {}",
                path.display()
            );
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid state file name: {}", path.display()))?;
        let temp_path = parent.join(format!(".{file_name}.tmp"));
        let file = create_private_file(&temp_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, value).context("failed to serialize state")?;
        writer
            .write_all(b"\n")
            .context("failed to terminate state file")?;
        writer.flush().context("failed to flush state file")?;
        writer
            .get_ref()
            .sync_all()
            .context("failed to sync state file")?;
        drop(writer);

        fs::rename(&temp_path, path).with_context(|| {
            format!(
                "failed to replace state file {} with {}",
                path.display(),
                temp_path.display()
            )
        })?;
        set_private_file_permissions(path)?;
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .context("failed to sync session directory")?;
        Ok(())
    }
}

pub fn validate_session_id(session_id: &str) -> Result<()> {
    let bytes = session_id.as_bytes();
    if bytes.is_empty() || bytes.len() > 128 {
        bail!("session id must be between 1 and 128 ASCII characters");
    }
    if !bytes[0].is_ascii_alphanumeric() {
        bail!("session id must start with an ASCII letter or digit");
    }
    if !bytes
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        bail!("session id may contain only ASCII letters, digits, '.', '_', and '-'");
    }

    let mut components = Path::new(session_id).components();
    let first = components.next();
    if !matches!(first, Some(Component::Normal(component)) if component == session_id)
        || components.next().is_some()
    {
        bail!("session id must be one normal path component");
    }

    Ok(())
}

pub fn schema_hash(schema: &Schema) -> String {
    use std::hash::{Hash, Hasher};
    use twox_hash::XxHash64;

    let mut hasher = XxHash64::default();
    canonical_schema_json(schema).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn canonical_schema_json(schema: &Schema) -> String {
    use serde_json::{Map, Value};

    let mut bits_map = Map::new();
    for (index, bit) in &schema.bits {
        let mut bit_object = Map::new();
        bit_object.insert("name".into(), Value::String(bit.name.clone()));
        bit_object.insert("desc".into(), Value::String(bit.desc.clone()));
        bits_map.insert(index.to_string(), Value::Object(bit_object));
    }

    let mut masks_map = Map::new();
    for (name, mask) in &schema.masks {
        let mut mask_object = Map::new();
        mask_object.insert(
            "bits".into(),
            Value::Array(
                mask.bits
                    .iter()
                    .map(|index| Value::Number((*index).into()))
                    .collect(),
            ),
        );
        mask_object.insert("desc".into(), Value::String(mask.desc.clone()));
        masks_map.insert(name.clone(), Value::Object(mask_object));
    }

    let mut root = Map::new();
    root.insert("version".into(), Value::Number(schema.version.into()));
    if let Some(default_mask) = &schema.default_mask {
        root.insert("default_mask".into(), Value::String(default_mask.clone()));
    }
    root.insert("bits".into(), Value::Object(bits_map));
    root.insert("masks".into(), Value::Object(masks_map));

    serde_json::to_string(&Value::Object(root)).expect("schema values are serializable")
}

fn read_json<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let file = File::open(path)?;
    serde_json::from_reader(BufReader::new(file)).context("invalid JSON state")
}

fn create_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn open_private_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    options.mode(0o600);
    let file = options.open(path)?;
    set_private_file_permissions(path)?;
    Ok(file)
}

fn create_private_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    let file = options.open(path)?;
    set_private_file_permissions(path)?;
    Ok(file)
}

fn set_private_file_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum LockMode {
    Shared,
    Exclusive,
}

struct SessionLock {
    file: File,
}

impl Drop for SessionLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::schema::{BitDef, MaskDef};
    use std::collections::BTreeMap;

    fn test_schema() -> Schema {
        Schema {
            version: 1,
            default_mask: None,
            bits: BTreeMap::from([
                (
                    0,
                    BitDef {
                        name: "auth".into(),
                        desc: "".into(),
                    },
                ),
                (
                    1,
                    BitDef {
                        name: "perm".into(),
                        desc: "".into(),
                    },
                ),
            ]),
            masks: BTreeMap::from([(
                "req".into(),
                MaskDef {
                    bits: vec![0, 1],
                    desc: "".into(),
                },
            )]),
        }
    }

    #[test]
    fn validates_session_ids() {
        for valid in ["a", "A0", "session.one_2-three"] {
            validate_session_id(valid).expect("id should be valid");
        }
        for invalid in ["", ".", "..", "-name", "_name", "a/b", r"a\b", "/tmp/x"] {
            assert!(
                validate_session_id(invalid).is_err(),
                "accepted {invalid:?}"
            );
        }
        assert!(validate_session_id(&"a".repeat(129)).is_err());
    }

    #[test]
    fn initializes_reads_updates_and_resets() {
        let temp = tempfile::tempdir().expect("temp directory should be created");
        let store = Store::new(temp.path().join("data")).expect("store should be created");
        let schema = test_schema();

        let initialized = store
            .initialize("session-1", &schema, false)
            .expect("session should initialize");
        assert_eq!(initialized.bits, 0);

        store
            .update_session("session-1", |_, session| {
                session.set_bit(1, true);
                Ok(())
            })
            .expect("session should update");
        let (_, loaded) = store
            .read_session("session-1")
            .expect("session should load");
        assert!(loaded.get_bit(1));

        let forced = store
            .initialize("session-1", &schema, true)
            .expect("session should reinitialize");
        assert_eq!(forced.bits, 0);
        assert!(store.reset("session-1").expect("session should reset"));
        assert!(
            !store
                .reset("session-1")
                .expect("reset should be idempotent")
        );
    }

    #[test]
    fn does_not_create_a_session_during_update() {
        let temp = tempfile::tempdir().expect("temp directory should be created");
        let store = Store::new(temp.path().join("data")).expect("store should be created");

        let result = store.update_session("missing", |_, _| Ok(()));
        assert!(result.is_err());
        assert!(!store.root().join("missing").exists());
    }

    #[test]
    fn detects_schema_hash_mismatch() {
        let temp = tempfile::tempdir().expect("temp directory should be created");
        let store = Store::new(temp.path().join("data")).expect("store should be created");
        store
            .initialize("session", &test_schema(), false)
            .expect("session should initialize");

        let session_path = store.root().join("session/session.json");
        let mut session: Session = read_json(&session_path).expect("session should parse");
        session.schema_hash = "wrong".into();
        store
            .write_json_atomic(&session_path, &session)
            .expect("fixture should be written");

        let error = store
            .read_session("session")
            .expect_err("mismatch must fail");
        assert!(error.to_string().contains("schema hash mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn applies_private_permissions() {
        let temp = tempfile::tempdir().expect("temp directory should be created");
        let store = Store::new(temp.path().join("data")).expect("store should be created");
        store
            .initialize("session", &test_schema(), false)
            .expect("session should initialize");

        let mode = |path: &Path| {
            fs::metadata(path)
                .expect("metadata should exist")
                .permissions()
                .mode()
                & 0o777
        };
        assert_eq!(mode(store.root()), 0o700);
        assert_eq!(mode(&store.root().join("session")), 0o700);
        assert_eq!(mode(&store.root().join("session/schema.json")), 0o600);
        assert_eq!(mode(&store.root().join("session/session.json")), 0o600);
    }
}
