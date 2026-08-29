use crate::models::Schema;
use crate::storage::Store;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct SetArgs {
    #[arg(long, help = "Session ID")]
    session: String,

    #[arg(long, help = "Bit name or index (comma-separated for multiple)")]
    bit: String,

    #[arg(
        long,
        help = "Value(s): true/false or 1/0 (comma-separated for multiple)"
    )]
    value: String,
}

fn parse_indices(schema: &Schema, bits_str: &str) -> Result<Vec<u8>> {
    let parts: Vec<&str> = bits_str.split(',').map(|s| s.trim()).collect();
    let mut indices = Vec::with_capacity(parts.len());

    for part in parts {
        if let Ok(idx) = part.parse::<u8>() {
            if idx > 63 {
                anyhow::bail!("bit index out of range (0-63): {idx}");
            }
            if !schema.bits.contains_key(&idx) {
                anyhow::bail!("bit index {idx} not defined in schema");
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
        anyhow::bail!("expected {expected_len} values, got {}", parts.len());
    }

    let mut values = Vec::with_capacity(parts.len());
    for part in parts {
        let val = match part.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => anyhow::bail!("invalid value '{part}': use true/false or 1/0"),
        };
        values.push(val);
    }
    Ok(values)
}

pub fn run(store: &Store, args: SetArgs) -> Result<()> {
    let updates = store.update_session(&args.session, |schema, session| {
        let indices = parse_indices(schema, &args.bit)?;
        let values = parse_values(&args.value, indices.len())?;
        session.set_bits(&indices, &values);

        Ok(indices
            .into_iter()
            .zip(values)
            .map(|(index, value)| (index, schema.bits[&index].name.clone(), value))
            .collect::<Vec<_>>())
    })?;

    for (index, name, value) in updates {
        println!("Set bit {index} ({name}) = {value}");
    }

    Ok(())
}
