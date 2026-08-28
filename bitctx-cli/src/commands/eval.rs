use crate::models::Schema;
use crate::storage::{load_schema, load_session, schema_hash};
use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;

#[derive(Args, Debug)]
pub struct EvalArgs {
    #[arg(long, help = "Session ID")]
    session: String,

    #[arg(long, help = "Mask name from schema")]
    mask: String,

    #[arg(long, default_value = "json", help = "Output format: json|text")]
    format: String,
}

#[derive(Serialize)]
pub struct EvalResult {
    pub pass: bool,
    pub missing: Vec<u8>,
    pub missing_labels: Vec<String>,
}

pub fn run(args: EvalArgs) -> Result<()> {
    let schema = load_schema(&args.session).context("failed to load schema")?;
    let hash = schema_hash(&schema);
    let session = load_session(&args.session, &hash).context("failed to load session")?;

    let mask_bits = schema.mask_bits(&args.mask).context("invalid mask")?;
    let (pass, missing_bits) = session.eval_mask(mask_bits);
    let missing_labels = schema.missing_labels(&args.mask, session.bits)?;

    let missing_indices: Vec<u8> = (0..64).filter(|&i| missing_bits & (1u64 << i) != 0).collect();

    let result = EvalResult {
        pass,
        missing: missing_indices,
        missing_labels,
    };

    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string(&result).unwrap());
        }
        "text" => {
            if pass {
                println!("PASS: all conditions satisfied");
            } else {
                println!("FAIL: missing conditions:");
                for (idx, label) in result.missing.iter().zip(result.missing_labels.iter()) {
                    println!("  - bit {}: {}", idx, label);
                }
            }
        }
        _ => anyhow::bail!("unknown format '{}': use json or text", args.format),
    }

    Ok(())
}