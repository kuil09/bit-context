use crate::models::Schema;
use crate::storage::{load_schema, load_or_create_session, save_session, schema_hash};
use anyhow::{Context, Result};
use clap::Args;

#[derive(Args, Debug)]
pub struct SetArgs {
    #[arg(long, help = "Session ID")]
    session: String,

    #[arg(long, help = "Bit name or index (comma-separated for multiple)")]
    bit: String,

    #[arg(long, help = "Value(s): true/false or 1/0 (comma-separated for multiple)")]
    value: String,
}

fn parse_indices(schema: &Schema, bits_str: &str) -> Result<Vec<u8>> {
    let parts: Vec<&str> = bits_str.split(',').map(|s| s.trim()).collect();
    let mut indices = Vec::with_capacity(parts.len());

    for part in parts {
        if let Ok(idx) = part.parse::<u8>() {
            if idx > 63 {
                anyhow::bail!("bit index out of range (0-63): {}", idx);
            }
            if !schema.bits.contains_key(&idx) {
                anyhow::bail!("bit index {} not defined in schema", idx);
            }
            indices.push(idx);
        } else {
            let idx = schema.bit_index(part)?;
            indices.push(idx);
        }
    }
    Ok(indices)
}

fn parse_values(values_str: &str, expected_len: usize) -> Result<Vec<bool>> {
    let parts: Vec<&str> = values_str.split(',').map(|s| s.trim()).collect();
    if parts.len() != expected_len {
        anyhow::bail!("expected {} values, got {}", expected_len, parts.len());
    }

    let mut values = Vec::with_capacity(parts.len());
    for part in parts {
        let val = match part.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => anyhow::bail!("invalid value '{}': use true/false or 1/0", part),
        };
        values.push(val);
    }
    Ok(values)
}

pub fn run(args: SetArgs) -> Result<()> {
    let schema = load_schema(&args.session).context("failed to load schema")?;
    let hash = schema_hash(&schema);
    let mut session = load_or_create_session(&args.session, &hash).context("failed to load or create session")?;

    let indices = parse_indices(&schema, &args.bit)?;
    let values = parse_values(&args.value, indices.len())?;

    session.set_bits(&indices, &values);
    save_session(&session).context("failed to save session")?;

    for (idx, val) in indices.iter().zip(values.iter()) {
        let name = schema.bits[idx].name.clone();
        println!("Set bit {} ({}) = {}", idx, name, val);
    }

    Ok(())
}