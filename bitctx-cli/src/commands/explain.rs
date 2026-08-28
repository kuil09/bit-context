use crate::models::Schema;
use crate::storage::{load_schema, load_session, schema_hash};
use anyhow::{Context, Result};
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

pub fn run(args: ExplainArgs) -> Result<()> {
    let schema = load_schema(&args.session).context("failed to load schema")?;
    let hash = schema_hash(&schema);
    let session = load_session(&args.session, &hash).context("failed to load session")?;

    let mask_bits = schema.mask_bits(&args.mask).context("invalid mask")?;
    let (pass, missing_bits) = session.eval_mask(mask_bits);

    if pass {
        match args.lang.as_str() {
            "ko" => println!("모든 조건이 충족되었습니다."),
            "en" => println!("All conditions satisfied."),
            _ => anyhow::bail!("unknown language '{}': use ko or en", args.lang),
        }
        return Ok(());
    }

    let missing_labels = schema.missing_labels(&args.mask, session.bits)?;
    let mask = schema.masks.get(&args.mask).unwrap();

    match args.lang.as_str() {
        "ko" => {
            println!("다음 조건이 충족되지 않았습니다 ({})", mask.desc);
            for label in missing_labels {
                println!("  - {}", label);
            }
        }
        "en" => {
            println!("Conditions not satisfied ({})", mask.desc);
            for label in missing_labels {
                println!("  - {}", label);
            }
        }
        _ => anyhow::bail!("unknown language '{}': use ko or en", args.lang),
    }

    Ok(())
}