use crate::commands::eval::evaluate;
use crate::models::{MissingCondition, Schema};
use crate::storage::Store;
use anyhow::Result;
use clap::Args;
use serde::Serialize;

const FRESHNESS: &str = "unverified";

#[derive(Args, Debug)]
pub struct ResumeArgs {
    #[arg(long, help = "Session ID")]
    session: String,

    #[arg(
        long,
        help = "Mask to resume (default: schema default_mask or the only mask)"
    )]
    mask: Option<String>,

    #[arg(long, default_value = "json", help = "Output format: json|text")]
    format: String,
}

#[derive(Serialize)]
pub struct ResumeResult {
    pub session_id: String,
    pub schema_hash: String,
    pub mask: String,
    pub pass: bool,
    pub missing: Vec<u8>,
    pub missing_labels: Vec<String>,
    pub missing_conditions: Vec<MissingCondition>,
    pub updated_at: String,
    pub freshness: &'static str,
}

pub fn run(store: &Store, args: ResumeArgs) -> Result<()> {
    let (schema, session) = store.read_session(&args.session)?;
    let mask = select_mask(&schema, args.mask.as_deref())?;
    let evaluation = evaluate(&schema, &mask, session.bits)?;
    let result = ResumeResult {
        session_id: session.id,
        schema_hash: session.schema_hash,
        mask,
        pass: evaluation.pass,
        missing: evaluation.missing,
        missing_labels: evaluation.missing_labels,
        missing_conditions: evaluation.missing_conditions,
        updated_at: session.updated_at,
        freshness: FRESHNESS,
    };

    match args.format.as_str() {
        "json" => println!("{}", serde_json::to_string(&result)?),
        "text" => render_text(&result),
        _ => anyhow::bail!("unknown format '{}': use json or text", args.format),
    }

    Ok(())
}

fn select_mask(schema: &Schema, requested: Option<&str>) -> Result<String> {
    if let Some(mask) = requested {
        if schema.masks.contains_key(mask) {
            return Ok(mask.to_string());
        }
        anyhow::bail!("mask '{mask}' not found in schema");
    }

    if let Some(mask) = &schema.default_mask {
        return Ok(mask.clone());
    }

    if schema.masks.len() == 1 {
        return Ok(schema
            .masks
            .keys()
            .next()
            .expect("one mask was counted")
            .clone());
    }

    let available = schema.masks.keys().cloned().collect::<Vec<_>>().join(", ");
    if available.is_empty() {
        anyhow::bail!("resume requires a mask, but the schema defines no masks");
    }
    anyhow::bail!(
        "resume requires --mask because the schema has multiple masks and no default_mask; available masks: {available}"
    );
}

fn render_text(result: &ResumeResult) {
    println!("Session: {}", result.session_id);
    println!("Mask: {}", result.mask);
    println!("Updated: {}", result.updated_at);
    println!("Freshness: {}", result.freshness);
    println!("RESULT: {}", if result.pass { 'O' } else { 'X' });
    println!("Missing:");
    if result.missing_conditions.is_empty() {
        println!("  (none)");
    } else {
        for condition in &result.missing_conditions {
            println!(
                "  X bit {}: {} ({})",
                condition.index, condition.name, condition.desc
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::schema::{BitDef, MaskDef};
    use std::collections::BTreeMap;

    fn schema(default_mask: Option<&str>, masks: &[&str]) -> Schema {
        Schema {
            version: 1,
            default_mask: default_mask.map(str::to_string),
            bits: BTreeMap::from([(
                0,
                BitDef {
                    name: "ready".into(),
                    desc: "Ready".into(),
                },
            )]),
            masks: masks
                .iter()
                .map(|name| {
                    (
                        (*name).to_string(),
                        MaskDef {
                            bits: vec![0],
                            desc: String::new(),
                        },
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn explicit_mask_overrides_default() {
        let schema = schema(Some("first"), &["first", "second"]);
        assert_eq!(
            select_mask(&schema, Some("second")).expect("explicit mask should resolve"),
            "second"
        );
    }

    #[test]
    fn default_and_single_masks_resolve() {
        assert_eq!(
            select_mask(&schema(Some("second"), &["first", "second"]), None)
                .expect("default mask should resolve"),
            "second"
        );
        assert_eq!(
            select_mask(&schema(None, &["only"]), None).expect("only mask should resolve"),
            "only"
        );
    }

    #[test]
    fn ambiguous_masks_require_selection() {
        let error = select_mask(&schema(None, &["first", "second"]), None)
            .expect_err("multiple masks should be ambiguous");
        let message = error.to_string();
        assert!(message.contains("requires --mask"));
        assert!(message.contains("first, second"));
    }
}
