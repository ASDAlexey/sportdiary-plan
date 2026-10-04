use anyhow::Result;
use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};
use runkit::{
    dates::{parse_iso_date, today_local},
    ingest, report, scaffold, validate,
};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "runkit",
    version,
    about = "Training log toolkit for the running repo"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        default_value = ".",
        help = "Repository root that contains data/ and reports/"
    )]
    root: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parse a GPX/TCX/JSON activity file into data/workouts
    Ingest(IngestArgs),
    /// Синхронизировать очищенный JSON истории Coros
    SyncCoros { path: PathBuf },
    /// Regenerate reports/DASHBOARD.md and reports/week-*.md
    Report,
    /// Schema-validate data/ and check training guardrails
    Validate,
    /// Plan week scaffolding
    Plan(PlanArgs),
}

#[derive(Args)]
struct IngestArgs {
    /// Activity file: .gpx, .tcx or .json
    path: PathBuf,
    /// Workout title
    #[arg(long)]
    title: Option<String>,
    /// Derive the title from the activity start time of day
    #[arg(long)]
    title_from_time: bool,
    /// Workout kind
    #[arg(long, default_value = "easy", value_enum)]
    kind: runkit::model::Kind,
    /// Data source tag; defaults to the file format
    #[arg(long)]
    source: Option<String>,
    /// Local date of the run, YYYY-MM-DD
    #[arg(long, value_parser = parse_date_cli)]
    date: Option<NaiveDate>,
    /// Subjective effort, 1-10
    #[arg(long)]
    rpe: Option<u8>,
    /// Free-text notes
    #[arg(long)]
    notes: Option<String>,
    /// Plan item this fulfills, e.g. 2026-W41/mon
    #[arg(long)]
    plan_ref: Option<String>,
}

#[derive(Args)]
struct PlanArgs {
    #[command(subcommand)]
    command: PlanCommand,
}

#[derive(Subcommand)]
enum PlanCommand {
    /// Scaffold data/plan/<isoweek>.yaml from the athlete weekly template
    New {
        /// Monday of the plan week, YYYY-MM-DD
        #[arg(long, value_parser = parse_date_cli)]
        monday: NaiveDate,
        /// Training phase
        #[arg(long, default_value = "base", value_enum)]
        phase: runkit::model::Phase,
        /// Overwrite an existing plan file
        #[arg(long)]
        force: bool,
    },
}

fn parse_date_cli(s: &str) -> Result<NaiveDate, String> {
    parse_iso_date(s).map_err(|e| e.to_string())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: &Cli) -> Result<ExitCode> {
    let root = &cli.root;
    match &cli.command {
        Command::SyncCoros { path } => {
            runkit::coros::run(root, path)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Ingest(args) => {
            let args = ingest::IngestArgs {
                path: args.path.clone(),
                title: args.title.clone(),
                title_from_time: args.title_from_time,
                kind: args.kind,
                source: args.source.clone(),
                date: args.date,
                rpe: args.rpe,
                notes: args.notes.clone(),
                plan_ref: args.plan_ref.clone(),
            };
            ingest::run(root, &args)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Report => {
            let outcome = report::run(root, today_local())?;
            if outcome.dashboard_written {
                println!("wrote reports/DASHBOARD.md");
            }
            let week_count = outcome.week_files_written.len();
            for f in &outcome.week_files_written {
                println!("wrote {f}");
            }
            if !outcome.dashboard_written && week_count == 0 {
                println!("reports are up to date");
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Validate => {
            let v = validate::run(root)?;
            validate::print(&v);
            if v.errors.is_empty() {
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::from(1))
            }
        }
        Command::Plan(plan) => match &plan.command {
            PlanCommand::New {
                monday,
                phase,
                force,
            } => {
                scaffold::run(root, *monday, *phase, *force)?;
                Ok(ExitCode::SUCCESS)
            }
        },
    }
}
