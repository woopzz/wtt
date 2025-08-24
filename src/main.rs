use std::collections::HashSet;

use chrono::{DateTime, Local as LocalTZ, NaiveDate, NaiveTime, TimeZone};
use clap::{Args, Parser, Subcommand};
use cli_table::{Cell, CellStruct, Style, Table};
use uuid::Uuid;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const DATE_FORMAT: &str = "%d.%m.%Y";
const DATETIME_FORMAT: &str = "%d.%m.%Y %H:%M";

#[derive(Parser)]
#[command(about=concat!(
    "A time tracker. Open a new session, do your job, close the session with a note.\n",
    "\n",
    "You can specify where to store the database via the environment variable WTT_PATH_DATABASE.",
))]
struct Cli {
    #[command(subcommand)]
    command: MainCommands,
}

#[derive(Subcommand)]
enum MainCommands {
    /// Manage sessions.
    Session(SessionArgs),
    /// Manage labels.
    Label(LabelArgs),
}

#[derive(Args)]
struct SessionArgs {
    #[command(subcommand)]
    command: SessionCommands,
}

#[derive(Subcommand)]
enum SessionCommands {
    /// Print a pretty representation of all sessions info.
    Pprint {
        /// Display the sessions which were created this day or later. The range is inclusive.
        #[arg(long, value_name = "dd.mm.yyyy or today")]
        from: Option<String>,
        /// Display the sessions which were created this day or earlier. The range is inclusive.
        #[arg(long, value_name = "dd.mm.yyyy")]
        to: Option<String>,
        /// Display the sessions which have at least one of these labels.
        #[arg(short, long)]
        labels: Vec<String>,
    },
    /// Create a new session.
    Create {
        /// A way to categorize sessions. You can provide several ones.
        #[arg(short, long)]
        labels: Vec<String>,
    },
    /// End a running session.
    End {
        /// A running session identifier. If not provided, the running session that was created last will be ended.
        #[arg(long)]
        id: Option<String>,
        /// Leave a message describing what you've done.
        #[arg(long)]
        note: Option<String>,
    },
    /// Add a note to a session.
    Note {
        /// A running session identifier.
        #[arg(long)]
        id: String,

        text: String,
    },
}

#[derive(Args)]
struct LabelArgs {
    #[command(subcommand)]
    command: LabelCommands,
}

#[derive(Subcommand)]
enum LabelCommands {
    /// Display all available labels.
    List {},
    /// Create a new label.
    Create {
        #[arg(long)]
        name: String,
    },
    /// Delete an existing label.
    Delete {
        #[arg(long)]
        name: String,
    },
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Store {
    sessions: Vec<Session>,
    labels: Vec<String>,
}

impl Store {
    fn get_all_sessions(
        &self,
        from_timestamp: Option<i64>,
        to_timestamp: Option<i64>,
        labels: &Vec<String>,
    ) -> Vec<&Session> {
        let labelset: HashSet<&str> = labels.iter().map(|x| x.as_str()).collect();
        let mut sessions: Vec<&Session> = self
            .sessions
            .iter()
            .filter(|session| {
                if let Some(ft) = from_timestamp
                    && ft > session.start_at
                {
                    return false;
                }

                if let Some(tt) = to_timestamp
                    && let Some(ttx) = session.end_at
                    && tt < ttx
                {
                    return false;
                }

                if labelset.len() > 0
                    && !session.labels.iter().any(|x| labelset.contains(x.as_str()))
                {
                    return false;
                }

                return true;
            })
            .collect();
        sessions.sort_by_key(|x| x.start_at);
        sessions
    }

    fn add_session(&mut self, labels: Vec<String>) -> Result<()> {
        let unknown_labels: Vec<&str> = labels
            .iter()
            .filter_map(|x| (!self.labels.contains(x)).then_some(x.as_str()))
            .collect();
        if unknown_labels.len() > 0 {
            return Err(format!(
                "A label with the name '{}' has been already created.",
                unknown_labels.join(", ")
            )
            .into());
        }
        let id = Uuid::new_v4();
        let now: DateTime<_> = LocalTZ::now();
        let session = Session {
            id: id.to_string(),
            start_at: now.timestamp(),
            end_at: None,
            note: None,
            labels: labels,
        };
        self.sessions.push(session);
        Ok(())
    }

    fn end_session(&mut self, id: Option<String>, note: Option<String>) -> Result<()> {
        let session: &mut Session = match id {
            Some(session_id) => {
                let session = self.get_session_by_id(&session_id)?;
                if session.end_at.is_some() {
                    return Err(format!("The session {session_id} has already ended.").into());
                }
                session
            }
            None => self.get_newest_running_session()?,
        };

        let now: DateTime<_> = LocalTZ::now();
        session.end_at = Some(now.timestamp());
        session.note = note;

        Ok(())
    }

    fn add_note(&mut self, id: String, note: String) -> Result<()> {
        let session = self.get_session_by_id(&id)?;
        session.note = Some(note);
        Ok(())
    }

    fn get_session_by_id(&mut self, id: &str) -> Result<&mut Session> {
        match self.sessions.iter_mut().find(|x| x.id == id) {
            Some(x) => Ok(x),
            None => Err(format!("The session {id} was not found.").into()),
        }
    }

    fn get_newest_running_session(&mut self) -> Result<&mut Session> {
        let mut running_session_info: Vec<&mut Session> = self
            .sessions
            .iter_mut()
            .filter_map(|x| x.end_at.is_none().then_some(x))
            .collect();
        running_session_info.sort_by_key(|x| x.start_at);
        match running_session_info.pop() {
            Some(x) => Ok(x),
            None => Err("There is no running session.".into()),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Session {
    id: String,
    start_at: i64,
    end_at: Option<i64>,
    note: Option<String>,
    labels: Vec<String>,
}

fn get_path_to_store_file() -> String {
    std::env::var("WTT_PATH_DATABASE").unwrap_or("db.json".to_string())
}

fn get_pprint_note_cell_maxlength() -> u16 {
    if let Ok(value_string) = std::env::var("WTT_PPRINT_NOTE_CELL_MAXLENGTH") {
        return value_string
            .parse()
            .expect("The value for WTT_PPRINT_NOTE_CELL_MAXLENGTH is not a valid number.");
    }
    40
}

fn load_store() -> Result<Store> {
    let path = get_path_to_store_file();
    let file = std::fs::File::open(&path)
        .map_err(|x| format!("Could not open the database file {}. {}", &path, x))?;
    let reader = std::io::BufReader::new(file);
    let store: Store = serde_json::from_reader(reader)
        .map_err(|x| format!("Could not parse the database file as a JSON data. {x}"))?;
    Ok(store)
}

fn dump_store(store: &Store) -> Result<()> {
    let path = get_path_to_store_file();
    let store_json = serde_json::to_string(store)
        .map_err(|x| format!("Could not create a JSON string from the store. {x}"))?;
    std::fs::write(&path, store_json).map_err(|x| {
        format!(
            "Could not dump the JSON string into the database file {}. {}",
            &path, x
        )
    })?;
    Ok(())
}

fn print_labels() {
    let store = load_store().unwrap();
    println!("{}", store.labels.join("\t"));
}

fn add_label(name: String) {
    let mut store = load_store().unwrap();

    if store.labels.contains(&name) {
        panic!(
            "A label with the name \"{}\" has been already created.",
            &name
        );
    }

    store.labels.push(name.clone());
    dump_store(&store).unwrap();

    println!("A new label \"{}\" is created.", &name);
}

fn delete_label(name: String) {
    let mut store = load_store().unwrap();

    if !store.labels.contains(&name) {
        panic!("The label \"{}\" was not found.", &name);
    }

    store.labels.retain(|x| *x != name);

    for session in &mut store.sessions {
        if session.labels.contains(&name) {
            session.labels.retain(|x| *x != name);
        }
    }

    dump_store(&store).unwrap();
    println!("The label \"{}\" was successfully deleted.", &name);
}

fn print_sessions(from: Option<String>, to: Option<String>, labels: Vec<String>) {
    let from_timestamp: Option<i64> = from.as_ref().and_then(|x| {
        if x == "today" {
            return Some(
                LocalTZ::now()
                    .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                    .unwrap()
                    .timestamp(),
            );
        }
        Some(get_datetime_from_date_str(x, NaiveTime::from_hms_opt(0, 0, 0).unwrap()).timestamp())
    });
    let to_timestamp: Option<i64> = to.as_ref().and_then(|x| {
        Some(
            get_datetime_from_date_str(x, NaiveTime::from_hms_opt(23, 59, 59).unwrap()).timestamp(),
        )
    });

    let store = load_store().unwrap();
    let sessions = store.get_all_sessions(from_timestamp, to_timestamp, &labels);

    let mut total_duration: u32 = 0;
    let mut rows: Vec<Vec<CellStruct>> = vec![];
    for session in sessions.into_iter() {
        let start_dt = LocalTZ.timestamp_opt(session.start_at, 0).unwrap();

        let mut end_string: Option<String> = None;
        let mut duration: u32 = 0;
        if let Some(end_at) = session.end_at {
            let end_dt = LocalTZ.timestamp_opt(end_at, 0).unwrap();
            let duration_delta = end_dt - start_dt;
            end_string = Some(end_dt.format(DATETIME_FORMAT).to_string());
            duration = duration_delta.num_minutes() as u32;
        }
        total_duration += duration;

        rows.push(vec![
            session.id.as_str().cell(),
            start_dt.format(DATETIME_FORMAT).cell(),
            session.labels.join(", ").cell(),
            match end_string {
                Some(x) => x.cell(),
                None => "".cell(),
            },
            format_duration(duration).cell(),
            match session.note {
                Some(ref x) => {
                    let cell_length = get_pprint_note_cell_maxlength() as usize;
                    let mut remainder: &str = x;
                    let mut tmp: &str;
                    let mut parts: Vec<&str> = vec![];
                    while remainder.len() > 0 {
                        if remainder.len() > cell_length {
                            (tmp, remainder) = remainder.split_at(cell_length - 1);
                            parts.push(tmp);
                        } else {
                            parts.push(remainder);
                            break;
                        }
                    }
                    parts.join("\n").cell()
                }
                None => "".cell(),
            },
        ])
    }
    let table = rows.table().title(vec![
        "ID".cell().bold(true),
        "Start".cell().bold(true),
        "Labels".cell().bold(true),
        "End".cell().bold(true),
        "Duration".cell().bold(true),
        "Note".cell().bold(true),
    ]);
    println!(
        "{}\nTotal duration of ended sessions: {}.",
        table
            .display()
            .expect("Could not build a table with sessions."),
        format_duration(total_duration),
    );
}

fn add_session(labels: Vec<String>) {
    let mut store = load_store().unwrap();
    store.add_session(labels).unwrap();
    dump_store(&store).unwrap();
}

fn end_session(id: Option<String>, note: Option<String>) {
    let mut store = load_store().unwrap();
    store.end_session(id, note).unwrap();
    dump_store(&store).unwrap();
}

fn add_note(id: String, text: String) {
    let mut store = load_store().unwrap();
    store.add_note(id, text).unwrap();
    dump_store(&store).unwrap();
}

fn get_datetime_from_date_str(date_str: &str, time: NaiveTime) -> DateTime<LocalTZ> {
    let date = NaiveDate::parse_from_str(date_str, "%d.%m.%Y").expect(&format!(
        "The date '{date_str}' must be provided in the format '{DATE_FORMAT}'."
    ));
    date.and_time(time).and_local_timezone(LocalTZ).unwrap()
}

fn format_duration(value: u32) -> String {
    let hours = value / 60;
    let minutes = value % 60;
    if hours > 0 {
        format!("{hours} hours {minutes} minutes")
    } else {
        format!("{minutes} minutes")
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        MainCommands::Session(session) => match session.command {
            SessionCommands::Pprint { from, to, labels } => print_sessions(from, to, labels),
            SessionCommands::Create { labels } => add_session(labels),
            SessionCommands::End { id, note } => end_session(id, note),
            SessionCommands::Note { id, text } => add_note(id, text),
        },
        MainCommands::Label(label) => match label.command {
            LabelCommands::List {} => print_labels(),
            LabelCommands::Create { name } => add_label(name),
            LabelCommands::Delete { name } => delete_label(name),
        },
    }
}
