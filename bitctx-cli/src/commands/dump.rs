use crate::storage::Store;
use anyhow::Result;
use clap::Args;
use serde::Serialize;

#[derive(Args, Debug)]
pub struct DumpArgs {
    #[arg(long, help = "Session ID")]
    session: String,

    #[arg(long, default_value = "text", help = "Output format: json|text")]
    format: String,
}

#[derive(Serialize)]
pub struct DumpResult {
    pub session_id: String,
    pub schema_hash: String,
    pub bits: u64,
    pub bit_states: Vec<BitState>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct BitState {
    pub index: u8,
    pub name: String,
    pub value: bool,
    pub desc: String,
}

pub fn run(store: &Store, args: DumpArgs) -> Result<()> {
    let (schema, session) = store.read_session(&args.session)?;

    let bit_states: Vec<BitState> = schema
        .all_bit_names()
        .into_iter()
        .map(|(idx, name)| {
            let desc = schema.bits[&idx].desc.clone();
            BitState {
                index: idx,
                name,
                value: session.get_bit(idx),
                desc,
            }
        })
        .collect();

    let result = DumpResult {
        session_id: session.id.clone(),
        schema_hash: session.schema_hash.clone(),
        bits: session.bits,
        bit_states,
        created_at: session.created_at.clone(),
        updated_at: session.updated_at.clone(),
    };

    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "text" => {
            println!("Session: {}", result.session_id);
            println!("Schema Hash: {}", result.schema_hash);
            println!("Bits: 0b{:064b} (0x{:016x})", result.bits, result.bits);
            println!("Created: {}", result.created_at);
            println!("Updated: {}", result.updated_at);
            println!();
            println!("Bit States:");
            for bs in &result.bit_states {
                let status = if bs.value { "●" } else { "○" };
                println!(
                    "  {} bit {:2}: {:20} = {}  ({})",
                    status, bs.index, bs.name, bs.value, bs.desc
                );
            }
        }
        _ => anyhow::bail!("unknown format '{}': use json or text", args.format),
    }

    Ok(())
}
