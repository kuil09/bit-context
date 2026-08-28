use crate::storage::delete_session;
use anyhow::{Context, Result};
use clap::Args;

#[derive(Args, Debug)]
pub struct ResetArgs {
    #[arg(long, help = "Session ID")]
    session: String,

    #[arg(long, default_value = "false", help = "Skip confirmation")]
    force: bool,
}

pub fn run(args: ResetArgs) -> Result<()> {
    if !args.force {
        print!("Delete session '{}'? [y/N] ", args.session);
        use std::io::{self, Write};
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    delete_session(&args.session).context("failed to delete session")?;
    println!("Session '{}' deleted.", args.session);
    Ok(())
}