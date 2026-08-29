use crate::models::MissingCondition;
use crate::storage::Store;
use anyhow::Result;
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
    pub missing_conditions: Vec<MissingCondition>,
}

pub fn run(store: &Store, args: EvalArgs) -> Result<()> {
    let (schema, session) = store.read_session(&args.session)?;
    let missing_conditions = schema.missing_conditions(&args.mask, session.bits)?;

    let result = EvalResult {
        pass: missing_conditions.is_empty(),
        missing: missing_conditions
            .iter()
            .map(|condition| condition.index)
            .collect(),
        missing_labels: missing_conditions
            .iter()
            .map(|condition| condition.name.clone())
            .collect(),
        missing_conditions,
    };

    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string(&result)?);
        }
        "text" => {
            if result.pass {
                println!("PASS: all conditions satisfied");
            } else {
                println!("FAIL: missing conditions:");
                for condition in &result.missing_conditions {
                    println!(
                        "  - bit {}: {} ({})",
                        condition.index, condition.name, condition.desc
                    );
                }
            }
        }
        _ => anyhow::bail!("unknown format '{}': use json or text", args.format),
    }

    Ok(())
}
