# Woopzzz's Time Tracker (WTT)

```bash
$ pip install -r requirements.txt
$ ./wtt
```

### How I use the app

I use virtualenv so it needs to be activated before running the app. For this purpose (for the most part) I have the following bash script.

```bash
#!/bin/sh
WTT_HOME=$HOME/wtt
. $WTT_HOME/.env/bin/activate
export WTT_PATH_DATABASE="$HOME/.local/share/wtt.json"
$WTT_HOME/wtt $@
```
