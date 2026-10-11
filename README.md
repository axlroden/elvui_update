<p align="center"><img src="assets/icon.png" width="128" alt=""></p>

<h1 align="center">elvui_update</h1>

<p align="center">Keeps ElvUI up to date in every World of Warcraft install on your machine.<br>Works on Windows, macOS and Linux.</p>

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
--wait <seconds>  Keep the window open when done (default: 60 on Windows and in the macOS app, 0 elsewhere)
--version         Print version and exit
```

## Download

Grab the binary for your platform from the [releases page](https://github.com/axlroden/elvui_update/releases):

- **Windows:** `elvui_update-windows.exe`. Double-click it.
- **macOS:** `elvui_update-macos-app.zip`. Unzip and move **ElvUI Updater** to Applications; it opens in Terminal when you start it. Universal (Intel and Apple Silicon).
- **Linux:** `elvui_update-linux.tar.gz`. Unpack and run `./install.sh` to get **ElvUI Updater** in your app menu (installs to `~/.local`). Static x86_64.

The bare binaries `elvui_update-macos` and `elvui_update-linux` are there too, for scripts and cron. Make them executable first: `chmod +x elvui_update-linux`.

macOS blocks unsigned downloads the first time. Allow it under System Settings > Privacy & Security ("Open Anyway"), or run `xattr -dr com.apple.quarantine "ElvUI Updater.app"`.

## Run it on a schedule

**Windows:** Task Scheduler > Create Basic Task, point it at the exe, and add `--wait 0` as an argument so it doesn't sit for a minute.

**macOS / Linux:** add a cron entry, e.g. `0 8 * * * /path/to/elvui_update`.

## Building

```
cargo build --release
```

Pushing a change to the version in `Cargo.toml` builds all three platforms on GitHub Actions and publishes a release. Pull requests get the same build without the release.

The icon lives in `assets/`. After editing `icon.svg` (or `icon-small.svg`, the simplified one for 16-32 px), regenerate the `.ico`, `.icns` and `.png` with `python assets/make_icons.py` (needs `pip install resvg-py pillow`).
