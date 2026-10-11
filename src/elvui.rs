//! Talking to the tukui API and installing ElvUI.

use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::time::Duration;

/// Written into the ElvUI folder after an install, so we know what we
/// installed even when the .toc only has a "@project-version@" placeholder.
const MARKER: &str = ".elvui_update_version";
/// Newest packaging first: ElvUI now ships one ElvUI.toc for every flavor.
const TOC_FILES: [&str; 2] = ["ElvUI.toc", "ElvUI_Mainline.toc"];

pub fn normalize(v: &str) -> String {
    v.trim().trim_start_matches(['v', 'V']).trim().to_string()
}

pub fn local_version(addons: &Path) -> String {
    let elv = addons.join("ElvUI");
    if let Ok(v) = fs::read_to_string(elv.join(MARKER)) {
        return normalize(&v);
    }
    for toc in TOC_FILES {
        let Ok(bytes) = fs::read(elv.join(toc)) else {
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        let found = text.lines().find_map(|l| {
            let l = l.trim_start_matches('#').trim_start();
            l.strip_prefix("Version:").map(str::trim)
        });
        match found {
            Some(v) if !v.contains('@') => return normalize(v),
            _ => break, // toc found but unusable: don't trust older leftover tocs
        }
    }
    if elv.exists() {
        "Unknown".into()
    } else {
        "Not Installed".into()
    }
}

pub struct Client {
    agent: ureq::Agent,
    pub api: String,
}

pub struct Release {
    pub version: String,
    pub url: String,
}

impl Client {
    pub fn new(api: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(20))
            .timeout_read(Duration::from_secs(60))
            .user_agent(concat!("elvui_update/", env!("CARGO_PKG_VERSION")))
            .build();
        Client { agent, api }
    }

    /// GET with a few retries so a network blip doesn't fail the run.
    fn get(&self, url: &str) -> Result<ureq::Response, String> {
        let mut last = String::new();
        for attempt in 1..=3u64 {
            if attempt > 1 {
                std::thread::sleep(Duration::from_secs(attempt * 3));
            }
            match self.agent.get(url).call() {
                Ok(r) => return Ok(r),
                Err(e) => last = e.to_string(),
            }
        }
        Err(last)
    }

    pub fn latest(&self) -> Result<Release, String> {
        let body = self
            .get(&format!("{}/addon/elvui", self.api))?
            .into_string()
            .map_err(|e| e.to_string())?;
        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| format!("unexpected API response: {e}"))?;
        let obj = if json.is_array() { &json[0] } else { &json }; // tolerate a list
        let field = |k: &str| match &obj[k] {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => String::new(),
        };
        let (version, url) = (normalize(&field("version")), field("url"));
        if version.is_empty() || url.is_empty() {
            return Err("API response missing version or url".into());
        }
        Ok(Release { version, url })
    }

    pub fn download(&self, url: &str) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        self.get(url)?
            .into_reader()
            .take(200 << 20)
            .read_to_end(&mut buf)
            .map_err(|e| e.to_string())?;
        Ok(buf)
    }

    /// Prints the newest version's entry from the changelog.
    pub fn print_changelog(&self) {
        let text = match self
            .get(&format!("{}/changelog/elvui", self.api))
            .and_then(|r| r.into_string().map_err(|e| e.to_string()))
        {
            Ok(t) => t,
            Err(e) => return println!("Could not fetch changelog: {e}"),
        };
        let mut in_version = false;
        for line in text.lines() {
            if line.starts_with("### Version") {
                if in_version {
                    break;
                }
                in_version = true;
                println!("{}", line.replace('\\', ""));
            } else if in_version && !line.trim().is_empty() {
                println!("{}", line.replace('\\', "").trim());
            }
        }
        println!();
    }
}

/// Clean install: the ElvUI folders in the zip are removed first so stale
/// files (old per-flavor .toc files, which WoW prefers over ElvUI.toc) don't
/// linger. Other addons are left alone.
pub fn install(zip_data: &[u8], addons: &Path, version: &str) -> Result<(), String> {
    let err = |e: &dyn std::fmt::Display| e.to_string();
    let mut zip = zip::ZipArchive::new(io::Cursor::new(zip_data)).map_err(|e| err(&e))?;
    fs::create_dir_all(addons).map_err(|e| err(&e))?;

    let mut top: Vec<String> = zip
        .file_names()
        .filter_map(|n| n.split_once('/').map(|(t, _)| t.to_string()))
        .filter(|t| t.starts_with("ElvUI"))
        .collect();
    top.sort();
    top.dedup();
    for dir in top {
        let target = addons.join(&dir);
        if target.is_dir() {
            fs::remove_dir_all(&target)
                .map_err(|e| format!("removing {}: {e}", target.display()))?;
        }
    }

    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(|e| err(&e))?;
        // enclosed_name rejects absolute paths and "..", guarding against zip-slip.
        let rel = f
            .enclosed_name()
            .ok_or_else(|| format!("zip contains unsafe path {:?}", f.name()))?;
        let dest = addons.join(rel);
        if f.is_dir() {
            fs::create_dir_all(&dest).map_err(|e| err(&e))?;
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| err(&e))?;
        }
        let mut out = fs::File::create(&dest).map_err(|e| format!("{}: {e}", dest.display()))?;
        io::copy(&mut f, &mut out).map_err(|e| err(&e))?;
    }
    fs::write(addons.join("ElvUI").join(MARKER), version).map_err(|e| err(&e))
}
