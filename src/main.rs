use clap::{Parser, Subcommand, Args};

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
    /// Manage labels.
    Label(LabelArgs),
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

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Session {
    id: String,
    start_at: i64,
    end_at: Option<i64>,
    note: Option<String>,
    labels: Vec<String>,
}

fn get_path_to_store_file() -> String {
    return std::env::var("WTT_PATH_DATABASE").unwrap_or("db.json".to_string());
}

fn load_store() -> Store {
    let path = get_path_to_store_file();
    let file = std::fs::File::open(&path).expect(&format!("Could not open the database file \"{}\".", &path));
    let reader = std::io::BufReader::new(file);
    let store: Store = serde_json::from_reader(reader).expect("Could not parse the database file as a JSON data.");
    return store;
}

fn dump_store(store: &Store) {
    let path = get_path_to_store_file();
    let store_json = serde_json::to_string(store).expect("Could not create a JSON string from the store.");
    std::fs::write(&path, store_json).expect(&format!("Could not dump the JSON string into the database file \"{}\".", &path));
}

fn print_labels() {
    let store = load_store();
    println!("{}", store.labels.join("\t"));
}

fn add_label(name: String) {
    let mut store = load_store();

    if store.labels.contains(&name) {
        panic!("A label with the name \"{}\" has been already created.", &name);
    }

    store.labels.push(name.clone());
    dump_store(&store);

    println!("A new label \"{}\" is created.", &name);
}

fn delete_label(name: String) {
    let mut store = load_store();

    if !store.labels.contains(&name) {
        panic!("The label \"{}\" was not found.", &name);
    }

    store.labels.retain(|x| *x != name);

    for session in &mut store.sessions {
        if session.labels.contains(&name) {
            session.labels.retain(|x| *x != name);
        }
    }

    dump_store(&store);
    println!("The label \"{}\" was successfully deleted.", &name);
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        MainCommands::Label(label) => {
            match label.command {
                LabelCommands::List {} => print_labels(),
                LabelCommands::Create { name } => add_label(name),
                LabelCommands::Delete { name } => delete_label(name),
            }
        }
    }
}
