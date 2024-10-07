use clap::{Args, Parser, Subcommand};
use rusqlite::Connection;
use std::{fs, io};
use vivaldi_history_extractor::{get_search_records, get_visit_records};

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Visit {
        #[command(flatten)]
        args: CommonArgs,
    },
    Search {
        #[command(flatten)]
        args: CommonArgs,
    },
}

#[derive(Clone, Args)]
struct CommonArgs {
    /// Path to the input file.
    #[arg(short, long)]
    #[arg(default_value = "History")]
    input: String,

    /// Path to the output file.
    #[arg(short, long)]
    #[arg(default_value = "result.json")]
    output: String,

    /// Overwrite the output file if it already exists.
    #[arg(long)]
    #[arg(default_value_t = false)]
    force: bool,
}

fn get_common_args(cmd: &Commands) -> CommonArgs {
    match cmd {
        Commands::Visit { args } => args.clone(),
        Commands::Search { args } => args.clone(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let common_args = get_common_args(&cli.command);

    // ex(Windows). r#"C:\Users\<username>\AppData\Local\Vivaldi\User Data\Default\History"#
    let path = common_args.input;

    let conn = if fs::exists(&path)? {
        Connection::open(path)?
    } else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "input file not found",
        )));
    };

    let out = if common_args.force {
        fs::File::create(common_args.output)?
    } else {
        fs::File::create_new(common_args.output)?
    };

    let items_num = match cli.command {
        Commands::Visit { args: _ } => {
            let visits = get_visit_records(conn)?;
            serde_json::to_writer_pretty(out, &visits)?;
            visits.len()
        }
        Commands::Search { args: _ } => {
            let searches = get_search_records(conn)?;
            serde_json::to_writer_pretty(out, &searches)?;
            searches.len()
        }
    };

    println!("items number: {items_num}");

    Ok(())
}
