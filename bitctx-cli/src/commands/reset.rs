use crate::storage::{Store, validate_session_id};
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct ResetArgs {
    #[arg(long, help = "Session ID")]
    session: String,

    #[arg(long, default_value = "false", help = "Skip confirmation")]
    force: bool,
}

pub fn run(store: &Store, args: ResetArgs) -> Result<()> {
    validate_session_id(&args.session)?;
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

    if store.reset(&args.session)? {
        println!("Session '{}' deleted.", args.session);
    } else {
        println!("Session '{}' did not exist.", args.session);
    }
    Ok(())
}
