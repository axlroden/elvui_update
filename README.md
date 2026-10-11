# elvui_update

Keeps ElvUI up to date in every World of Warcraft install on your machine. Works on Windows, macOS and Linux.

## What it does

- Reads Battle.net's install list to find your WoW installs (retail, classic, classic era, beta, anything Battle.net lists).
- Asks the tukui API for the latest ElvUI version and compares it with what's installed.
- If they differ, it downloads ElvUI once and does a clean install into each install's `Interface/AddOns`. The old ElvUI folders are removed first; other addons are left alone.
- Prints the latest changelog entry.

On Linux there is no native Battle.net, so it looks inside Wine prefixes (`$WINEPREFIX`, `~/.wine`, Lutris, Steam/Proton and Bottles).

If it can't find your install, point it at the AddOns folder directly:

```
elvui_update --path "/path/to/World of Warcraft/_retail_/Interface/AddOns"
```

## Options

```
--path <dir>      AddOns folder to update (repeatable); skips Battle.net detection
--force           Reinstall even if already up to date
--no-changelog    Don't print the changelog
--wait <seconds>  Keep the window open when done (default: 60 on Windows, 0 elsewhere)
--version         Print version and exit
```

## Download

Grab the binary for your platform from the [releases page](https://github.com/axlroden/elvui_update/releases):

- `elvui_update-windows.exe`
- `elvui_update-macos` (universal: Intel and Apple Silicon)
- `elvui_update-linux` (static, x86_64)

On macOS and Linux, make it executable first: `chmod +x elvui_update-macos`. macOS will block an unsigned download the first time; allow it under System Settings > Privacy & Security, or run `xattr -d com.apple.quarantine elvui_update-macos`.

## Run it on a schedule

**Windows:** Task Scheduler > Create Basic Task, point it at the exe, and add `--wait 0` as an argument so it doesn't sit for a minute.

**macOS / Linux:** add a cron entry, e.g. `0 8 * * * /path/to/elvui_update`.

## Building

```
cargo build --release
```

Pushing a change to the version in `Cargo.toml` builds all three platforms on GitHub Actions and publishes a release.
