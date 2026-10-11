//! elvui_update keeps ElvUI up to date in every World of Warcraft install
//! Battle.net knows about, on Windows, macOS and Linux (Wine/Lutris/Proton).

mod detect;
mod elvui;
mod productdb;

use detect::Target;
use std::path::PathBuf;
use std::time::Duration;

const USAGE: &str = "\
Usage: elvui_update [options]

Updates ElvUI in every World of Warcraft install Battle.net knows about.

Options:
  --path <dir>      AddOns folder to update (repeatable); skips Battle.net detection
  --force           Reinstall even if already up to date
  --no-changelog    Don't print the changelog
  --wait <seconds>  Keep the window open when done (default: 60 on Windows and in
                    the macOS app, 0 elsewhere)
  --api <url>       tukui API base URL
  --version         Print version and exit
  -h, --help        Show this help";

struct Opts {
    paths: Vec<PathBuf>,
    force: bool,
    changelog: bool,
    wait: u64,
    api: String,
}

fn parse_args() -> Result<Opts, String> {
    let mut o = Opts {
        paths: Vec::new(),
        force: false,
        changelog: true,
        wait: if cfg!(windows) || in_mac_app() { 60 } else { 0 },
        api: "https://api.tukui.org/v1".into(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let (key, inline) = match a.split_once('=') {
            Some((k, v)) => (k.to_string(), Some(v.to_string())),
            None => (a.clone(), None),
        };
        let mut value = || {
            inline
                .clone()
                .or_else(|| args.next())
                .ok_or(format!("{key} needs a value"))
        };
        match key.trim_start_matches('-') {
            "path" => o.paths.push(value()?.into()),
            "force" => o.force = true,
            "no-changelog" => o.changelog = false,
            "wait" => {
                o.wait = value()?
                    .parse()
                    .map_err(|_| "--wait needs a number of seconds")?
            }
            "api" => o.api = value()?,
            "version" => {
                println!("elvui_update {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "h" | "help" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            _ => return Err(format!("unknown option {a}\n\n{USAGE}")),
        }
    }
    Ok(o)
}

/// Running from inside "ElvUI Updater.app" rather than as a bare binary.
fn in_mac_app() -> bool {
    cfg!(target_os = "macos")
        && std::env::current_exe()
            .is_ok_and(|exe| exe.to_string_lossy().contains(".app/Contents/MacOS/"))
}

/// Started from Finder, the app has no terminal to print to, so open one and
/// run there instead.
fn reopen_in_terminal() {
    use std::io::IsTerminal;
    if !in_mac_app() || std::io::stdout().is_terminal() || std::env::args().len() > 1 {
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let opened = std::process::Command::new("open")
        .args(["-a", "Terminal"])
        .arg(exe)
        .status()
        .is_ok_and(|s| s.success());
    if opened {
        std::process::exit(0);
    }
}

fn main() {
    reopen_in_terminal();
    let (code, wait) = match parse_args() {
        Ok(o) => {
            let wait = o.wait;
            match run(o) {
                Ok(()) => (0, wait),
                Err(e) => {
                    eprintln!("\nERROR: {e}");
                    (1, wait)
                }
            }
        }
        Err(e) => {
            eprintln!("{e}");
            (2, 0)
        }
    };
    std::thread::sleep(Duration::from_secs(wait));
    std::process::exit(code);
}

fn run(o: Opts) -> Result<(), String> {
    let targets = if o.paths.is_empty() {
        detect::find_targets()?
    } else {
        o.paths
            .into_iter()
            .map(|addons| Target {
                label: "custom".into(),
                addons,
            })
            .collect()
    };
    if targets.is_empty() {
        println!("No World of Warcraft installs found in Battle.net.");
        return Ok(());
    }

    let client = elvui::Client::new(o.api);
    let rel = client
        .latest()
        .map_err(|e| format!("checking latest ElvUI version: {e}"))?;

    let mut zip: Option<Vec<u8>> = None; // download once, install everywhere
    let mut failed = 0;
    for t in &targets {
        let local = elvui::local_version(&t.addons);
        println!("[{}] {}", t.label, t.addons.display());
        println!("  Installed: {local}\n  Latest:    {}", rel.version);
        if local == rel.version && !o.force {
            println!("  Up to date");
            continue;
        }
        if zip.is_none() {
            println!("  Downloading..");
            zip = Some(
                client
                    .download(&rel.url)
                    .map_err(|e| format!("downloading ElvUI: {e}"))?,
            );
        }
        match elvui::install(zip.as_deref().unwrap(), &t.addons, &rel.version) {
            Ok(()) => println!("  Updated"),
            Err(e) => {
                println!("  Install FAILED: {e}");
                failed += 1;
            }
        }
    }
    println!();

    if o.changelog {
        client.print_changelog();
    }
    if failed > 0 {
        return Err(format!("{failed} install(s) failed"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
