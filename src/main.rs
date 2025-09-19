use std::{collections::HashSet, fs};

use chrono::{DateTime, Local as LocalTZ, NaiveDate, NaiveTime, TimeDelta, TimeZone};
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
    /// Display all sessions in a table format.
    Table {
        /// Display the sessions which were started this day or later. The range is inclusive.
        #[arg(long, value_name = "dd.mm.yyyy or today")]
        from: Option<String>,
        /// Display the sessions which were started this day or earlier. The range is inclusive.
        #[arg(long, value_name = "dd.mm.yyyy")]
        to: Option<String>,
        /// Display the sessions which have at least one of these labels.
        #[arg(short, long)]
        labels: Vec<String>,
    },
    /// Start a new session.
    Start {
        /// A way to categorize sessions. You can provide several ones.
        #[arg(short, long)]
        labels: Vec<String>,
    },
    /// End a running session.
    End {
        /// A running session identifier. If not provided, the running session that was started last will be ended.
        #[arg(long)]
        id: Option<String>,
        /// Leave a message describing what you've done.
        #[arg(long)]
        note: Option<String>,
    },
    /// Update the note of a session.
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
    /// Display a list of all labels.
    List {},
    /// Remove a label from all sessions.
    Remove { name: String },
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Store {
    sessions: Vec<Session>,
}

impl Store {
    fn from_store_file() -> Result<Self> {
        let path = get_path_to_store_file();

        let file_exists = fs::exists(&path)
            .map_err(|x| format!("Could not check the database file {}. {}", &path, x))?;
        if !file_exists {
            return Ok(Self { sessions: vec![] });
        }

        let file = std::fs::File::open(&path)
            .map_err(|x| format!("Could not open the database file {}. {}", &path, x))?;
        let reader = std::io::BufReader::new(file);
        let store: Store = serde_json::from_reader(reader)
            .map_err(|x| format!("Could not parse the database file as JSON data. {x}"))?;
        Ok(store)
    }

    fn save(&self) -> Result<()> {
        let path = get_path_to_store_file();
        let store_json = serde_json::to_string(self)
            .map_err(|x| format!("Could not create a JSON string from the store. {x}"))?;
        std::fs::write(&path, store_json).map_err(|x| {
            format!(
                "Could not dump the JSON string into the database file {}. {}",
                &path, x
            )
        })?;
        Ok(())
    }

    fn get_all_sessions(
        &self,
        from_timestamp: Option<i64>,
        to_timestamp: Option<i64>,
        labels: &[String],
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

    fn start_session(&mut self, labels: Vec<String>) -> Result<&Session> {
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
        Ok(self.sessions.last().unwrap())
    }

    fn end_session(&mut self, id: Option<&str>, note: Option<String>) -> Result<&Session> {
        let session: &mut Session = match id {
            Some(session_id) => {
                let session = self.get_session_by_id(session_id)?;
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

        Ok(session)
    }

    fn update_note(&mut self, id: &str, note: String) -> Result<()> {
        let session = self.get_session_by_id(id)?;
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

    fn get_all_labels(&self) -> HashSet<&str> {
        self.sessions
            .iter()
            .flat_map(|x| &x.labels)
            .map(|x| x.as_str())
            .collect::<HashSet<&str>>()
    }

    fn remove_label(&mut self, name: &str) -> Result<u32> {
        let mut count: u32 = 0;
        for session in &mut self.sessions {
            let count_before = session.labels.len();
            session.labels.retain(|x| *x != name);
            count += u32::try_from(count_before - session.labels.len()).unwrap();
        }
        Ok(count)
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
            .expect("The value for WTT_PPRINT_NOTE_CELL_MAXLENGTH is not a valid u16 number.");
    }
    40
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

    let store = Store::from_store_file().unwrap();
    let sessions = store.get_all_sessions(from_timestamp, to_timestamp, &labels);

    let mut total_duration: u32 = 0;
    let mut rows: Vec<Vec<CellStruct>> = vec![];
    let now = LocalTZ::now();
    for session in sessions.into_iter() {
        let start_dt = LocalTZ.timestamp_opt(session.start_at, 0).unwrap();

        let mut end_string: Option<String> = None;
        let duration_delta: TimeDelta;
        if let Some(end_at) = session.end_at {
            let end_dt = LocalTZ.timestamp_opt(end_at, 0).unwrap();
            end_string = Some(end_dt.format(DATETIME_FORMAT).to_string());
            duration_delta = end_dt - start_dt;
        } else {
            duration_delta = now - start_dt;
        }
        let duration = duration_delta.num_minutes() as u32;
        total_duration += duration;

        rows.push(vec![
            session.id.as_str().cell(),
            start_dt.format(DATETIME_FORMAT).cell(),
            session.labels.join(", ").cell(),
            match end_string {
                Some(x) => x.cell(),
                None => "".cell(),
            },
            format_duration(duration, session.end_at.is_none(), "\n").cell(),
            match session.note {
                Some(ref x) => {
                    let max_width = get_pprint_note_cell_maxlength();
                    built_multilined_note(x, usize::from(max_width)).cell()
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
        "{}\nTotal duration: {}.",
        table
            .display()
            .expect("Could not build a table with sessions."),
        format_duration(total_duration, false, " "),
    );
}

fn get_datetime_from_date_str(date_str: &str, time: NaiveTime) -> DateTime<LocalTZ> {
    let date = NaiveDate::parse_from_str(date_str, "%d.%m.%Y").expect(&format!(
        "The date '{date_str}' must be provided in the format '{DATE_FORMAT}'."
    ));
    date.and_time(time).and_local_timezone(LocalTZ).unwrap()
}

fn format_duration(value: u32, still_running: bool, separator: &str) -> String {
    let mut parts: Vec<String> = vec![];

    if still_running {
        parts.push("for now".to_string());
    }

    let hours = value / 60;
    if hours > 0 {
        parts.push(format!("{hours} hours"));
    }

    let minutes = value % 60;
    parts.push(format!("{minutes} minutes"));

    return parts.join(separator);
}

fn built_multilined_note(text: &str, max_width: usize) -> String {
    let mut text: &str = text;
    let mut tmp: &str;
    let mut parts: Vec<&str> = vec![];
    while text.len() > 0 {
        if text.len() <= max_width {
            parts.push(text);
            break;
        }

        let mut last_whitespace_index: Option<usize> = None;
        for (index, char) in text.char_indices() {
            if index >= max_width {
                break;
            }
            if char == ' ' {
                last_whitespace_index = Some(index);
            }
        }

        if let Some(ws_index) = last_whitespace_index {
            (tmp, text) = text.split_at(ws_index);
            text = text.trim_start();
            parts.push(tmp);
        } else {
            (tmp, text) = text.split_at(max_width - 1);
            parts.push(tmp);
        }
    }
    parts.join("\n")
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        MainCommands::Session(session) => match session.command {
            SessionCommands::Table { from, to, labels } => print_sessions(from, to, labels),
            SessionCommands::Start { labels } => {
                let mut store = Store::from_store_file().unwrap();
                let session = store.start_session(labels).unwrap();
                println!("New session was successfully started: {}", &session.id);
                store.save().unwrap();
            }
            SessionCommands::End { id, note } => {
                let mut store = Store::from_store_file().unwrap();
                let session = store.end_session(id.as_deref(), note).unwrap();
                println!("The session {} was successfully ended.", &session.id);
                store.save().unwrap();
            }
            SessionCommands::Note { id, text } => {
                let mut store = Store::from_store_file().unwrap();
                store.update_note(&id, text).unwrap();
                println!("Updated.");
                store.save().unwrap();
            }
        },
        MainCommands::Label(label) => match label.command {
            LabelCommands::List {} => {
                let store = Store::from_store_file().unwrap();
                let labels = store.get_all_labels();
                println!("{}", labels.into_iter().collect::<Vec<&str>>().join("\n"));
            }
            LabelCommands::Remove { name } => {
                let mut store = Store::from_store_file().unwrap();
                let removed_count = store.remove_label(&name).unwrap();
                store.save().unwrap();
                println!("Removed {} labels.", removed_count);
            }
        },
    }
}
