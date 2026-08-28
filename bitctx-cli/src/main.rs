use clap::{Parser, Subcommand};
use commands::{dump, eval, explain, init, reset, set};

mod commands;
mod models;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "bitctx", version, about = "Bit-memory context store for AI harness skills")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init(init::InitArgs),
    Set(set::SetArgs),
    Eval(eval::EvalArgs),
    Explain(explain::ExplainArgs),
    Dump(dump::DumpArgs),
    Reset(reset::ResetArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => init::run(args),
        Commands::Set(args) => set::run(args),
        Commands::Eval(args) => eval::run(args),
        Commands::Explain(args) => explain::run(args),
        Commands::Dump(args) => dump::run(args),
        Commands::Reset(args) => reset::run(args),
    }
}