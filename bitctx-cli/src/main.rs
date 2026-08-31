use clap::{Parser, Subcommand};
use commands::{dump, eval, explain, init, reset, resume, set};
use std::path::PathBuf;
use storage::Store;

mod commands;
mod models;
mod storage;

#[derive(Parser, Debug)]
#[command(
    name = "bitctx",
    version,
    about = "Bit-memory context store for AI harness skills"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        env = "BITCTX_DATA_DIR",
        value_name = "PATH",
        help = "Session data directory (env: BITCTX_DATA_DIR; default: ~/.bitctx)"
    )]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a session from a schema.
    Init(init::InitArgs),
    /// Set one or more condition bits.
    Set(set::SetArgs),
    /// Evaluate a named mask.
    Eval(eval::EvalArgs),
    /// Resume decision state without replaying completed work.
    Resume(resume::ResumeArgs),
    /// Explain missing conditions.
    Explain(explain::ExplainArgs),
    /// Dump the complete session state.
    Dump(dump::DumpArgs),
    /// Delete a session.
    Reset(reset::ResetArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let store = Store::from_data_dir(cli.data_dir)?;

    match cli.command {
        Commands::Init(args) => init::run(&store, args),
        Commands::Set(args) => set::run(&store, args),
        Commands::Eval(args) => eval::run(&store, args),
        Commands::Resume(args) => resume::run(&store, args),
        Commands::Explain(args) => explain::run(&store, args),
        Commands::Dump(args) => dump::run(&store, args),
        Commands::Reset(args) => reset::run(&store, args),
    }
}
