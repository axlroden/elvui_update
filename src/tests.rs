use crate::{detect, elvui, productdb, run, Opts};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::{fs, thread};

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("elvui_test_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn make_zip(files: &[(&str, &str)]) -> Vec<u8> {
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, body) in files {
        w.start_file(*name, opts).unwrap();
        w.write_all(body.as_bytes()).unwrap();
    }
    w.finish().unwrap().into_inner()
}

fn write(p: &Path, s: &str) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, s).unwrap();
}

/// Set ELVUI_TEST_PRODUCTDB to a real product.db to check parsing against it.
#[test]
fn real_product_db() {
    let Some(p) = std::env::var_os("ELVUI_TEST_PRODUCTDB") else {
        return;
    };
    let installs = productdb::read(Path::new(&p)).unwrap();
    assert!(!installs.is_empty(), "no WoW installs parsed");
    for i in &installs {
        println!("{i:?}");
        assert!(
            !i.path.is_empty() && !i.extra.is_empty(),
            "incomplete: {i:?}"
        );
    }
}

#[test]
fn corrupt_product_db_is_an_error_not_a_crash() {
    assert!(productdb::parse(&[0x0a, 0xff]).is_err());
    assert!(productdb::parse(&[0x0a, 0x05, 0x01]).is_err());
}

// Wine prefixes only exist on Linux, and "d:" isn't a valid folder name on Windows.
#[cfg(unix)]
#[test]
fn wine_paths() {
    let prefix = tmp("wine");
    fs::create_dir_all(prefix.join("drive_c")).unwrap();
    fs::create_dir_all(prefix.join("dosdevices/d:")).unwrap();
    assert_eq!(
        detect::host_path("D:/games/World of Warcraft", Some(&prefix)).unwrap(),
        prefix.join("dosdevices/d:/games/World of Warcraft")
    );
    assert_eq!(
        detect::host_path("C:/Program Files (x86)/World of Warcraft", Some(&prefix)).unwrap(),
        prefix.join("drive_c/Program Files (x86)/World of Warcraft")
    );
    assert!(detect::host_path("Z:/x", Some(&prefix)).is_none());
}

#[test]
fn toc_versions() {
    let a = tmp("toc");
    assert_eq!(elvui::local_version(&a), "Not Installed");
    write(
        &a.join("ElvUI/ElvUI_Mainline.toc"),
        "## Title: ElvUI\n## Version: v15.26\n",
    );
    assert_eq!(elvui::local_version(&a), "15.26");
    // New single toc with an unexpanded placeholder: don't trust the stale Mainline toc.
    write(
        &a.join("ElvUI/ElvUI.toc"),
        "## Version: @project-version@\n",
    );
    assert_eq!(elvui::local_version(&a), "Unknown");
}

#[test]
fn zip_slip_is_rejected() {
    let a = tmp("slip");
    let z = make_zip(&[("ElvUI/../../evil.txt", "x")]);
    assert!(elvui::install(&z, &a.join("AddOns"), "1").is_err());
    assert!(!a.join("evil.txt").exists());
}

/// Tiny HTTP server standing in for the tukui API.
fn serve(zip: Vec<u8>) -> String {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    let b2 = base.clone();
    thread::spawn(move || {
        for s in l.incoming() {
            let mut s = s.unwrap();
            let mut buf = [0u8; 4096];
            let n = s.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]);
            let path = req.split_whitespace().nth(1).unwrap_or("").to_string();
            let body: Vec<u8> = match path.as_str() {
                "/addon/elvui" => {
                    format!(r#"{{"version":"v15.30","url":"{b2}/elvui.zip"}}"#).into_bytes()
                }
                "/elvui.zip" => zip.clone(),
                "/changelog/elvui" => {
                    b"### Version 15.30 [ Oct 1 ]\n* one\n\n* two\n### Version 15.29\n* old\n"
                        .to_vec()
                }
                _ => Vec::new(),
            };
            let status = if body.is_empty() {
                "404 Not Found"
            } else {
                "200 OK"
            };
            let _ = write!(
                s,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = s.write_all(&body);
        }
    });
    base
}

#[test]
fn end_to_end() {
    let zip = make_zip(&[
        (
            "ElvUI/ElvUI.toc",
            "## Interface: 120100\n## Version: @project-version@\n",
        ),
        ("ElvUI/Core/init.lua", "-- core"),
        ("ElvUI_Libraries/libs.lua", "-- libs"),
        ("ElvUI_Options/options.lua", "-- options"),
    ]);
    let api = serve(zip);
    let a = tmp("e2e");
    write(&a.join("ElvUI/ElvUI_Mainline.toc"), "## Version: v15.26\n");
    write(&a.join("DBM-Core/x.lua"), "");

    let opts = |changelog| Opts {
        paths: vec![a.clone()],
        force: false,
        changelog,
        wait: 0,
        api: api.clone(),
    };
    run(opts(true)).unwrap();
    assert!(
        !a.join("ElvUI/ElvUI_Mainline.toc").exists(),
        "stale toc left behind"
    );
    assert!(a.join("DBM-Core/x.lua").exists(), "other addon removed");
    assert!(a.join("ElvUI_Options/options.lua").exists());
    assert_eq!(elvui::local_version(&a), "15.30");
    run(opts(false)).unwrap(); // second run: up to date
}
