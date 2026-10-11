//! Finds WoW installs via Battle.net's product.db on Windows, macOS and
//! Linux (inside Wine, Lutris, Steam/Proton or Bottles prefixes).

use crate::productdb;
use std::path::{Path, PathBuf};

/// One AddOns folder to keep ElvUI updated in.
pub struct Target {
    pub label: String,
    pub addons: PathBuf,
}

/// A product.db, plus the Wine prefix it lives in (Linux only).
struct AgentDb {
    path: PathBuf,
    prefix: Option<PathBuf>,
}

pub fn find_targets() -> Result<Vec<Target>, String> {
    let dbs = product_dbs();
    if dbs.is_empty() {
        return Err(
            "could not find Battle.net's product.db (use --path to point at your AddOns folder)"
                .into(),
        );
    }
    let mut targets: Vec<Target> = Vec::new();
    for db in dbs {
        let installs = match productdb::read(&db.path) {
            Ok(i) => i,
            Err(e) => {
                println!("Skipping {}: {e}", db.path.display());
                continue;
            }
        };
        for inst in installs {
            let Some(root) = host_path(&inst.path, db.prefix.as_deref()) else {
                continue;
            };
            let flavor = root.join(&inst.extra);
            if !flavor.is_dir() {
                continue;
            }
            let addons = flavor.join("Interface").join("AddOns");
            if targets.iter().any(|t| t.addons == addons) {
                continue;
            }
            targets.push(Target {
                label: inst.code,
                addons,
            });
        }
    }
    targets.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(targets)
}

fn product_dbs() -> Vec<AgentDb> {
    let mut candidates: Vec<AgentDb> = Vec::new();
    if cfg!(windows) {
        let pd = std::env::var_os("ALLUSERSPROFILE").unwrap_or_else(|| r"C:\ProgramData".into());
        candidates.push(AgentDb {
            path: PathBuf::from(pd).join(r"Battle.net\Agent\product.db"),
            prefix: None,
        });
    } else if cfg!(target_os = "macos") {
        candidates.push(AgentDb {
            path: "/Users/Shared/Battle.net/Agent/product.db".into(),
            prefix: None,
        });
    } else {
        for prefix in wine_prefixes() {
            let path = prefix.join("drive_c/ProgramData/Battle.net/Agent/product.db");
            candidates.push(AgentDb {
                path,
                prefix: Some(prefix),
            });
        }
    }
    candidates.retain(|c| c.path.is_file());
    candidates
}

fn wine_prefixes() -> Vec<PathBuf> {
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let mut out = Vec::new();
    if let Some(p) = std::env::var_os("WINEPREFIX") {
        out.push(PathBuf::from(p));
    }
    out.push(home.join(".wine"));
    // (parent folder, suffix inside each child) for launchers that keep many prefixes
    let parents: [(&str, &str); 7] = [
        ("Games", ""), // Lutris default
        (".local/share/lutris/prefixes", ""),
        (".steam/steam/steamapps/compatdata", "pfx"), // Steam/Proton
        (".local/share/Steam/steamapps/compatdata", "pfx"),
        (
            ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata",
            "pfx",
        ),
        (".local/share/bottles/bottles", ""), // Bottles
        (".var/app/com.usebottles.bottles/data/bottles/bottles", ""),
    ];
    for (parent, suffix) in parents {
        if let Ok(entries) = std::fs::read_dir(home.join(parent)) {
            for e in entries.flatten() {
                out.push(e.path().join(suffix));
            }
        }
    }
    out
}

/// Converts a product.db install path into a path on this machine. Inside a
/// Wine prefix, "D:/games/WoW" becomes <prefix>/dosdevices/d:/games/WoW.
pub fn host_path(p: &str, prefix: Option<&Path>) -> Option<PathBuf> {
    let Some(prefix) = prefix else {
        return Some(PathBuf::from(p));
    };
    let p = p.replace('\\', "/");
    let b = p.as_bytes();
    if b.len() < 2 || b[1] != b':' {
        return None;
    }
    let drive = p[..2].to_ascii_lowercase();
    let rest = p[2..].trim_start_matches('/');
    let dev = prefix.join("dosdevices").join(&drive);
    if dev.exists() {
        Some(dev.join(rest))
    } else if drive == "c:" {
        Some(prefix.join("drive_c").join(rest))
    } else {
        None
    }
}
