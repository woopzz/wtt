#[derive(serde::Deserialize, Debug)]
struct Store {
    sessions: Vec<Session>,
    labels: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
struct Session {
    id: String,
    start_at: i64,
    end_at: Option<i64>,
    note: Option<String>,
    labels: Vec<String>,
}

fn load_store<P: AsRef<std::path::Path> + std::fmt::Display>(path: P) -> Store {
    let file = std::fs::File::open(&path).expect(&format!("Could not open the database file \"{}\".", &path));
    let reader = std::io::BufReader::new(file);
    let store: Store = serde_json::from_reader(reader).expect("Could not parse the database file as a JSON file.");
    return store;
}

fn main() {
    let path_to_store_file = std::env::var("WTT_PATH_DATABASE").unwrap_or("db.json".to_string());
    let store = load_store(path_to_store_file);
    println!("{:?}", store);
}
