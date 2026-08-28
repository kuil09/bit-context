use crate::models::Schema;
use crate::storage::{save_schema, schema_hash, session_dir};
use anyhow::{Context, Result};
use clap::Args;
use std::fs;
use std::path::Path;

#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(long, help = "Session ID")]
    session: String,

    #[arg(long, help = "Path to schema JSON file")]
    schema: String,

    #[arg(long, default_value = "false", help = "Overwrite existing session")]
    force: bool,
}

pub fn run(args: InitArgs) -> Result<()> {
    let schema_path = Path::new(&args.schema);
    if !schema_path.exists() {
        anyhow::bail!("schema file not found: {}", schema_path.display());
    }

    let schema_content = fs::read_to_string(schema_path).context("failed to read schema file")?;
    let mut schema: Schema = serde_json::from_str(&schema_content).context("invalid schema JSON")?;
    schema.validate().context("schema validation failed")?;

    let dir = session_dir(&args.session);
    if dir.exists() && !args.force {
        anyhow::bail!(
            "session '{}' already exists. Use --force to overwrite",
            args.session
        );
    }

    fs::create_dir_all(&dir).context("failed to create session directory")?;

    let hash = schema_hash(&schema);
    save_schema(&args.session, &schema).context("failed to save schema")?;

    println!("Initialized session '{}' with schema hash {}", args.session, hash);
    Ok(())
}