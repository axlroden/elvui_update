#!/bin/sh
# Installs elvui_update for the current user, with an app menu entry and icon.
set -e
cd "$(dirname "$0")"

bin="${XDG_BIN_HOME:-$HOME/.local/bin}"
data="${XDG_DATA_HOME:-$HOME/.local/share}"

install -Dm755 elvui_update "$bin/elvui_update"
install -Dm644 elvui_update.png "$data/icons/hicolor/512x512/apps/elvui_update.png"
install -Dm644 elvui_update.svg "$data/icons/hicolor/scalable/apps/elvui_update.svg"
mkdir -p "$data/applications"
sed "s|^Exec=elvui_update|Exec=\"$bin/elvui_update\"|" elvui_update.desktop \
    > "$data/applications/elvui_update.desktop"

# Refresh menu and icon caches where the desktop keeps them.
update-desktop-database "$data/applications" 2>/dev/null || true
gtk-update-icon-cache -q -t "$data/icons/hicolor" 2>/dev/null || true

echo "Installed $bin/elvui_update"
echo "ElvUI Updater is now in your app menu."
