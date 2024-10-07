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
    #[arg(default_value = default_input_path())]
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

fn default_input_path() -> impl clap::builder::IntoResettable<clap::builder::OsStr> {
    // example
    // Windows: C:\Users\<username>\AppData\Local\Vivaldi\User Data\Default\History
    // Linux:   /home/<username>/.config/vivaldi/Default/History

    let default: String = String::from("History");

    #[cfg(not(any(windows, target_os = "linux")))]
    {
        default
    }

    #[cfg(any(windows, target_os = "linux"))]
    {
        use std::path::PathBuf;

        let Some(user_dir) = directories::UserDirs::new() else {
            return default;
        };
        let Some(home) = user_dir.home_dir().to_str() else {
            return default;
        };

        #[cfg(windows)]
        {
            let history_path: PathBuf = [
                home,
                "AppData",
                "Local",
                "Vivaldi",
                "User Data",
                "Default",
                "History",
            ]
            .iter()
            .collect();
            let Some(history_path) = history_path.to_str() else {
                return default;
            };
            String::from(history_path)
        }

        #[cfg(target_os = "linux")]
        {
            let history_path: PathBuf = [home, ".config", "vivaldi", "Default", "History"]
                .iter()
                .collect();
            let Some(history_path) = history_path.to_str() else {
                return default;
            };
            String::from(history_path)
        }
    }
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
