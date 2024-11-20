mod cli;
mod globals;
mod helpers;
mod parse;
mod server;
mod shell;
mod structs;
mod table;
mod task;

use clap::{Parser, ValueEnum};
use clap_verbosity_flag::Verbosity;
use macros_rs::str;
use std::path::Path;

macro_rules! dispatch {
    ($cli:expr, { $($flag:ident => $func:expr),+ $(,)? }) => {$(
        if $cli.$flag {
            return $func;
        }
    )+};
}

#[derive(Parser)]
#[command(version = str!(cli::get_version(false)))]
#[clap(disable_help_flag = true, disable_help_subcommand = true)]
struct Cli {
    /// Run a task defined in maidfile
    #[arg(default_value = "", hide_default_value = true)]
    task: Vec<String>,
    
    /// Base path for Maidfile
    #[arg(short, long, default_value = "maidfile")]
    path: String,
    
    /// Ignore cache on build
    #[arg(short, long)]
    force: bool,
    
    /// Switch Maid to server mode
    #[arg(short, long, visible_alias = "online")]
    remote: bool,
    
    /// Clear build cache
    #[arg(short = 'C', long, visible_alias = "purge", group = "commands")]
    clean_cache: bool,
    
    /// Create new Maid project
    #[arg(short, long, group = "commands")]
    init: bool,
    
    /// List all runnable tasks
    #[arg(short, long, visible_alias = "tasks", visible_alias = "ls", group = "commands")]
    list: bool,
    
    /// Watch for changes in specified path 
    #[arg(short, long)]
    watch: Option<String>,
    
    /// View Maid health (server health if enabled)
    #[arg(short = 'H', long, group = "commands")]
    health: bool,
    
    /// Per project commands
    #[arg(short = 'w', long, group = "commands")]
    project: Option<Project>,
    
    /// Management Maid commands
    #[arg(short = 'g', long, group = "commands")]
    system: Option<System>,
    
    #[clap(flatten)]
    verbose: Verbosity,
    
    /// Shows this quick reference
    #[clap(short, long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
}

#[derive(ValueEnum, Clone)]
enum System {
    /// Check for new Maid updates
    Update,
    /// Return the Maidfile in json
    Json,
    /// Hydrate json output with environment
    HydrateJson,
}

#[derive(ValueEnum, Clone)]
enum Project {
    /// Get Project Info
    Info,
}

fn main() {
    let cli = Cli::parse();

    globals::init();
    env_logger::Builder::new().filter_level(cli.verbose.log_level_filter()).init();
    
    dispatch!(cli, {
        init => cli::butler::init(),
        health => server::cli::connect(&cli.path), 
        health => match cli.remote {
            true => server::cli::connect(&cli.path),
            false => server::cli::connect(&cli.path), // improve health command for later
        },
        clean_cache => match cli.remote {
            true => server::cli::connect(&cli.path),
            false => cli::butler::clean(),
        },
        list => match cli.remote {
            true => cli::tasks::List::remote(&cli.path, cli.verbose.is_silent(), cli.verbose.log_level()),
            false => cli::tasks::List::all(&cli.path, cli.verbose.is_silent(), cli.verbose.log_level(), cli.force),
        }
    });
    
    if let Some(project) = cli.project {
        return match project {
            Project::Info => cli::info(&cli.path), // add more info
        };
    }
    
    if let Some(system) = cli.system {
        return match system {
            System::Update => cli::butler::update(), // add real update checker
            System::Json => cli::tasks::json(&cli.path, &cli.task, false),
            System::HydrateJson => cli::tasks::json(&cli.path, &cli.task, true),
        };
    }
    
    if let Some(path) = cli.watch {
        return cli::butler::watch(Path::new(&path)); // migrate watch path into executer below
    }
    
    cli::exec(cli.task[0].trim(), &cli.task, &cli.path, cli.verbose.is_silent(), false, cli.remote, cli.verbose.log_level(), cli.force)
}
