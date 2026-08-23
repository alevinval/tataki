use std::process::ExitCode;

use chrono::Local;
use chrono::TimeDelta;
use clap::Parser;
use clap::Subcommand;
use thiserror::Error;
use tt_lib::Scheduler;
use tt_lib::StorageError;
use tt_lib::Store;
use tt_lib::parser;
use tt_lib::types::Blueprint;
use tt_lib::types::experimental::book::Book;
use tt_lib::types::experimental::journal::Journal;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Compute and print the plan for the next 7 days
    Schedule,
    /// Initialise a new tataki project in the current directory
    Init,
    /// List the blueprints in the book
    List,
    /// Add a blueprint to the book
    #[command(
        after_help = r#"Blueprint line format: {id} {priority} {recurrence} {duration} {availability}

Availability formats:
  {hours}                e.g. 08:00-12:00
  {days}                 e.g. Mon-Fri
  {days} {hours}         e.g. Mon-Fri 08:00-12:00

Examples:
  tt-cli add "1 NORM daily 1h 08:00-12:00" "Deep work"
  tt-cli add "2 HIGH weekly 30min Mon-Fri" "Team standup"
  tt-cli add "3 CRIT ^1d 2h Mon-Fri 08:00-12:00" "Focus block""#
    )]
    Add {
        /// Blueprint line, e.g. `1 NORM ^1d 1h Mon-Fri 08:00-12:00`
        blueprint: String,
        /// Human-readable description
        description: String,
    },
    /// Remove a blueprint from the book by id
    Remove {
        /// Id of the blueprint to remove
        id: String,
    },
}

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("{0}")]
    Parse(String),
    #[error("blueprint with id '{0}' already exists")]
    DuplicateId(String),
    #[error("no blueprint with id '{0}'")]
    NotFound(String),
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), Error> {
    match Cli::parse().command {
        Some(Command::Schedule) => run_schedule(),
        Some(Command::Init) => run_init(),
        Some(Command::List) => run_list(),
        Some(Command::Add {
            blueprint,
            description,
        }) => run_add(blueprint, description),
        Some(Command::Remove { id }) => run_remove(id),
        None => Ok(()),
    }
}

fn run_init() -> Result<(), Error> {
    let cwd = std::env::current_dir().map_err(StorageError::CurrentDir)?;
    if Store::open_default().is_some() {
        return Err(Error::Storage(StorageError::AlreadyInitialised {
            path: cwd.join(".tataki"),
        }));
    }
    let store = Store::init(cwd)?;
    Book::new(Vec::new()).save(&store)?;
    println!("Initialised tataki project at {}", store.root().display());
    Ok(())
}

fn run_list() -> Result<(), Error> {
    let store = open_store()?;
    let book = Book::load(&store)?;
    for bp in book.blueprints() {
        println!("{bp} {}", bp.description());
    }
    Ok(())
}

fn run_add(blueprint: String, description: String) -> Result<(), Error> {
    let store = open_store()?;
    let book = Book::load(&store)?;
    let (id, priority, recurrence, duration, availability) =
        parser::blueprint(&blueprint).map_err(Error::Parse)?;
    if book.blueprints().iter().any(|b| b.id() == id) {
        return Err(Error::DuplicateId(id));
    }
    let blueprint = Blueprint::new(
        id,
        description,
        duration,
        priority,
        recurrence,
        availability,
    );
    println!("Added blueprint '{}'", blueprint.id());
    let mut blueprints = book.blueprints().to_vec();
    blueprints.push(blueprint);
    Book::new(blueprints).save(&store)?;
    Ok(())
}

fn run_remove(id: String) -> Result<(), Error> {
    let store = open_store()?;
    let book = Book::load(&store)?;
    let blueprints: Vec<_> = book
        .blueprints()
        .iter()
        .filter(|b| b.id() != id.as_str())
        .cloned()
        .collect();
    if blueprints.len() == book.blueprints().len() {
        return Err(Error::NotFound(id));
    }
    Book::new(blueprints).save(&store)?;
    println!("Removed blueprint '{id}'");
    Ok(())
}

fn run_schedule() -> Result<(), Error> {
    let store = open_store()?;
    let book = Book::load(&store)?;
    let journal = Journal::load(&store)?;

    let now = Local::now();
    let report =
        Scheduler::new(book, journal).schedule_with_warnings(now, now + TimeDelta::days(7));
    for item in report.items() {
        println!("{item}");
    }

    Ok(())
}

fn open_store() -> Result<Store, Error> {
    Store::open_default()
        .ok_or(StorageError::NotInitialised)
        .map_err(Error::Storage)
}
