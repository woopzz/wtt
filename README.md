# Woopzzz's Time Tracker (WTT)

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
alias wttt="wtt session pprint --from today"
```
