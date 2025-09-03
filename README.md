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

```bash
$ cargo build --release
$ WTT_PATH_DATABASE=~/.local/share/wtt.json ./target/release/wtt --help
```

### How I use the app

I have the following script to avoid specifying the path to the store file every time.

```bash
#!/bin/bash
export WTT_PATH_DATABASE="$HOME/.local/share/wtt.json"
$HOME/repos/wtt/target/release/wtt "$@"
```

Also I have an alias to the most used command in my Bash config.

```bash
alias wttt="wtt session table --from today"
```
