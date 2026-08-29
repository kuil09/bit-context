use crate::storage::Store;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct ExplainArgs {
    #[arg(long, help = "Session ID")]
    session: String,

    #[arg(long, help = "Mask name from schema")]
    mask: String,

    #[arg(long, default_value = "ko", help = "Language: ko|en")]
    lang: String,
}

pub fn run(store: &Store, args: ExplainArgs) -> Result<()> {
    let (schema, session) = store.read_session(&args.session)?;
    let missing_conditions = schema.missing_conditions(&args.mask, session.bits)?;

    if missing_conditions.is_empty() {
        match args.lang.as_str() {
            "ko" => println!("모든 조건이 충족되었습니다."),
            "en" => println!("All conditions satisfied."),
            _ => anyhow::bail!("unknown language '{}': use ko or en", args.lang),
        }
        return Ok(());
    }

    let mask = schema
        .masks
        .get(&args.mask)
        .ok_or_else(|| anyhow::anyhow!("mask '{}' not found in schema", args.mask))?;

    match args.lang.as_str() {
        "ko" => {
            println!("다음 조건이 충족되지 않았습니다 ({})", mask.desc);
            for condition in &missing_conditions {
                println!("  - {}: {}", condition.name, condition.desc);
            }
        }
        "en" => {
            println!("Conditions not satisfied ({})", mask.desc);
            for condition in &missing_conditions {
                println!("  - {}: {}", condition.name, condition.desc);
            }
        }
        _ => anyhow::bail!("unknown language '{}': use ko or en", args.lang),
    }

    Ok(())
}
