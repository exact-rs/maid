mod cli;
mod globals;
mod parse;
mod server;
mod shell;
mod task;

use maid::log::{
    layer::prelude::*,
    verbose::{InfoLevel, Verbosity},
};

use clap::{Parser, ValueEnum};
use macros_rs::fmt::str;
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
    /// Run a task defined in Maidfile
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
    #[arg(short = 'W', long)]
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
    verbose: Verbosity<InfoLevel>,

    /// Shows this quick reference
    #[clap(short, long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
}

#[derive(ValueEnum, Clone)]
enum System {
    /// Check for new Maid updates
    CheckUpdates,
    /// Upgrade Maid to the latest version
    Upgrade,
    /// Return the Maidfile in json
    Json,
    /// Hydrate json with environment fields
    JsonHydrated,
}

#[derive(ValueEnum, Clone)]
enum Project {
    /// Retrieve project metadata
    Info,
    /// Display current defined environment
    Env,
}

fn main() {
    let cli = Cli::parse();
    let log_layer = MaidFormatLayer::new();

    globals::init();

    tracing_subscriber::registry().with(cli.verbose.log_level_filter()).with(log_layer).init();

    dispatch!(cli, {
        init => cli::dispatch::init(),
        health => server::cli::connect(&cli.path),
        health => match cli.remote {
            true => server::cli::connect(&cli.path),
            false => {}, // improve health command for later
        },
        clean_cache => match cli.remote {
            true => server::cli::connect(&cli.path),
            false => cli::dispatch::clean(),
        },
        list => match cli.remote {
            true => cli::tasks::list_remote(&cli.path, cli.verbose.is_silent(), cli.verbose.log_level()),
            false => cli::tasks::list_all(&cli.path, cli.verbose.is_silent(), cli.verbose.log_level(), cli.force),
        }
    });

    if let Some(project) = cli.project {
        return match project {
            Project::Info => cli::info(&cli.path), // add more info
            Project::Env => cli::env(&cli.path),
        };
    }

    if let Some(system) = cli.system {
        return match system {
            System::CheckUpdates => cli::dispatch::check_update(),
            System::Upgrade => {} // add upgrader
            System::Json => cli::tasks::list_json(&cli.path, &cli.task, false),
            System::JsonHydrated => cli::tasks::list_json(&cli.path, &cli.task, true),
        };
    }

    if let Some(path) = cli.watch {
        return cli::dispatch::watch(Path::new(&path)); // migrate watch path into executer below
    }

    cli::exec(
        cli.task[0].trim(),
        &cli.task,
        &cli.path,
        cli.verbose.is_silent(),
        false,
        cli.remote,
        cli.verbose.log_level(),
        cli.force,
        false,
    )
}
