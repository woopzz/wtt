# Woopzzz's Time Tracker (WTT)

A CLI app that helps you track work sessions. The workflow is straightforward:
1. Start a new session.
2. Work.
3. End the session with a short note about what you've done.

### Features

- End any running session by its ID. If no ID is provided, the app ends the most recently started session.
- Update the note of any session by its ID.
- Use labels to organize and differentiate your sessions.
- View all of your sessions in a table format with support for filtering by date or label.
- No pause / resume features (by design). From experience, it's better to end a session and rest, rather than falling into an endless pause / resume cicle.

### How to install

You'll need the standard Rust toolchain to build the app. [Go install it.](https://www.rust-lang.org/tools/install)

1. Clone the repository.
2. Go to the project's root folder.
3. Build the app:
```bash
$ cargo build --release
```
3. Run the app:
```bash
$ WTT_PATH_DATABASE=~/.local/share/wtt.json ./target/release/wtt --help
```

#### What is WTT_PATH_DATABASE

The environment variable "WTT_PATH_DATABASE" tells the app where to store your sessions and labels.
If you set the variable, the data will be saved to the specified file.
Otherwise, the app will default to "db.json" in the current folder.

#### How I use the app

I have the following script to avoid specifying the path to the store file every time.

```bash
#!/bin/bash
export WTT_PATH_DATABASE="$HOME/.local/share/wtt.json"
$HOME/repos/wtt/target/release/wtt "$@"
```

Make this script executable (`chmod +x name_of_the_script`) and put it into a folder which is mentioned in $PATH.

Also I have the following alias in my Bash config.

```bash
alias wttt="wtt session table --from today"
```

#### How to use

```bash
# Display all of today's sessions. You will probably use this often,
# so I suggest creating an alias for the command.
$ wtt session table --from today

# Start new session with a label.
$ wtt session start -l personal-project

# Do your work.
# Work..
# Work...

# End the last session. Add a note about what you did.
$ wtt session end --note "Did ..."

# View today's sessions again to see the completed entry.
$ wtt session table --from today

# Expore the help commands to see all available options.
$ wtt --help
$ wtt session --help
$ wtt session note --help
```
