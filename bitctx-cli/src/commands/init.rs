use crate::models::Schema;
use crate::storage::Store;
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

pub fn run(store: &Store, args: InitArgs) -> Result<()> {
    let schema_path = Path::new(&args.schema);
    if !schema_path.exists() {
        anyhow::bail!("schema file not found: {}", schema_path.display());
    }

    let schema_content = fs::read_to_string(schema_path).context("failed to read schema file")?;
    let schema: Schema = serde_json::from_str(&schema_content).context("invalid schema JSON")?;
    schema.validate().context("schema validation failed")?;
    let session = store
        .initialize(&args.session, &schema, args.force)
        .context("failed to initialize session")?;

    println!(
        "Initialized session '{}' with schema hash {}",
        args.session, session.schema_hash
    );
    Ok(())
}
