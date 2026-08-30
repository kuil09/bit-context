use crate::models::{MissingCondition, Schema};
use crate::storage::Store;
use anyhow::Result;
use clap::{Args, ValueEnum};
use serde::Serialize;
use std::fmt::Write;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ShowFilter {
    All,
    Satisfied,
    Missing,
}

impl ShowFilter {
    fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Satisfied => "satisfied",
            Self::Missing => "missing",
        }
    }
}

#[derive(Args, Debug)]
pub struct EvalArgs {
    #[arg(long, help = "Session ID")]
    session: String,

    #[arg(long, help = "Mask name from schema")]
    mask: String,

    #[arg(long, default_value = "json", help = "Output format: json|text")]
    format: String,

    #[arg(
        long,
        value_enum,
        help = "Show condition details in text output: all|satisfied|missing"
    )]
    show: Option<ShowFilter>,
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
            if args.show.is_some() {
                anyhow::bail!("--show can only be used with --format text");
            }
            println!("{}", serde_json::to_string(&result)?);
        }
        "text" => {
            let mask = schema
                .masks
                .get(&args.mask)
                .expect("mask was validated while computing missing conditions");
            print!(
                "{}",
                render_text(&schema, &mask.bits, session.bits, result.pass, args.show)
            );
        }
        _ => anyhow::bail!("unknown format '{}': use json or text", args.format),
    }

    Ok(())
}

fn render_text(
    schema: &Schema,
    mask_bits: &[u8],
    current_bits: u64,
    pass: bool,
    show: Option<ShowFilter>,
) -> String {
    let mut selected = [false; 64];
    for &index in mask_bits {
        selected[index as usize] = true;
    }

    let mut output = String::from("     0   1   2   3   4   5   6   7\n");
    writeln!(output, "00 ┌───┬───┬───┬───┬───┬───┬───┬───┐")
        .expect("writing to a String cannot fail");

    for row in 0..8 {
        output.push_str("   │");
        for column in 0..8 {
            let index = row * 8 + column;
            let marker = if !selected[index] {
                '·'
            } else if current_bits & (1_u64 << index) != 0 {
                'O'
            } else {
                'X'
            };
            write!(output, " {marker} │").expect("writing to a String cannot fail");
        }
        output.push('\n');

        if row < 7 {
            writeln!(
                output,
                "{:02} ├───┼───┼───┼───┼───┼───┼───┼───┤",
                (row + 1) * 8
            )
            .expect("writing to a String cannot fail");
        }
    }

    output.push_str("   └───┴───┴───┴───┴───┴───┴───┴───┘\n\n");
    writeln!(output, "RESULT: {}", if pass { 'O' } else { 'X' })
        .expect("writing to a String cannot fail");

    if let Some(filter) = show {
        writeln!(output, "\nDETAILS ({})", filter.label())
            .expect("writing to a String cannot fail");

        let mut shown = 0;
        for &index in mask_bits {
            let satisfied = current_bits & (1_u64 << index) != 0;
            let include = match filter {
                ShowFilter::All => true,
                ShowFilter::Satisfied => satisfied,
                ShowFilter::Missing => !satisfied,
            };
            if include {
                let bit = &schema.bits[&index];
                writeln!(
                    output,
                    "  {} bit {}: {} ({})",
                    if satisfied { 'O' } else { 'X' },
                    index,
                    bit.name,
                    bit.desc
                )
                .expect("writing to a String cannot fail");
                shown += 1;
            }
        }

        if shown == 0 {
            output.push_str("  (none)\n");
        }
    }

    output
}
