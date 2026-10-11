//! Embeds the icon and version info into the Windows .exe, so Explorer, the
//! taskbar and the file's Properties show them. The .res format is simple
//! enough to write by hand, which saves pulling in a resource compiler crate;
//! the MSVC linker takes the .res file directly.

use std::env;
use std::fs;
use std::path::PathBuf;

const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;
const RT_VERSION: u16 = 16;
const LANG_EN_US: u16 = 0x0409;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if os != "windows" || target_env != "msvc" {
        return;
    }
    let ico = fs::read("assets/icon.ico").expect("reading assets/icon.ico");
    let res = PathBuf::from(env::var("OUT_DIR").unwrap()).join("resources.res");
    fs::write(&res, resources(&ico)).expect("writing resources.res");
    println!("cargo:rustc-link-arg-bins={}", res.display());
}

fn resources(ico: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    entry(&mut out, 0, 0, 0, &[]); // every .res file starts with an empty entry

    // Each image in the .ico becomes an RT_ICON; the RT_GROUP_ICON is the .ico
    // directory with file offsets swapped for those resource ids.
    let count = u16_at(ico, 4);
    let mut group = Vec::new();
    for v in [0, 1, count] {
        group.extend(v.to_le_bytes());
    }
    for i in 0..count {
        let e = &ico[6 + 16 * i as usize..][..16];
        let size = u32_at(e, 8);
        let offset = u32_at(e, 12) as usize;
        let id = i + 1;
        entry(&mut out, RT_ICON, id, LANG_EN_US, &ico[offset..offset + size as usize]);
        group.extend(&e[..4]); // width, height, colour count, reserved
        group.extend(1u16.to_le_bytes()); // planes
        group.extend(u16_at(e, 6).max(32).to_le_bytes()); // bits per pixel
        group.extend(size.to_le_bytes());
        group.extend(id.to_le_bytes());
    }
    entry(&mut out, RT_GROUP_ICON, 1, LANG_EN_US, &group);
    entry(&mut out, RT_VERSION, 1, LANG_EN_US, &version_info());
    out
}

/// One resource: header with numeric type and name, then the data, DWORD aligned.
fn entry(out: &mut Vec<u8>, kind: u16, name: u16, lang: u16, data: &[u8]) {
    let flags: u16 = if kind == 0 { 0 } else { 0x1030 }; // moveable, pure, discardable
    out.extend((data.len() as u32).to_le_bytes());
    out.extend(32u32.to_le_bytes()); // header size
    for v in [0xFFFF, kind, 0xFFFF, name] {
        out.extend(v.to_le_bytes());
    }
    out.extend(0u32.to_le_bytes()); // data version
    out.extend(flags.to_le_bytes());
    out.extend(lang.to_le_bytes());
    out.extend(0u32.to_le_bytes()); // version
    out.extend(0u32.to_le_bytes()); // characteristics
    out.extend(data);
    pad(out);
}

fn version_info() -> Vec<u8> {
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let mut parts = version
        .split(|c: char| !c.is_ascii_digit())
        .map(|p| p.parse::<u32>().unwrap_or(0));
    let mut next = || parts.next().unwrap_or(0);
    let (major, minor, patch) = (next(), next(), next());
    let ms = major << 16 | minor;
    let ls = patch << 16;

    let fixed: Vec<u8> = [
        0xFEEF04BD, // signature
        0x0001_0000, // struct version
        ms, ls, // file version
        ms, ls, // product version
        0x3F, 0, // flags mask, flags
        0x0004_0004, // VOS_NT_WINDOWS32
        1, 0, // VFT_APP, no subtype
        0, 0, // file date
    ]
    .iter()
    .flat_map(|v: &u32| v.to_le_bytes())
    .collect();

    let description = env::var("CARGO_PKG_DESCRIPTION").unwrap();
    let strings = [
        ("FileDescription", description.as_str()),
        ("FileVersion", version.as_str()),
        ("InternalName", "elvui_update"),
        ("OriginalFilename", "elvui_update.exe"),
        ("ProductName", "ElvUI Updater"),
        ("ProductVersion", version.as_str()),
        ("LegalCopyright", "GPL-3.0"),
    ]
    .iter()
    .map(|(k, v)| {
        let value = utf16z(v);
        node(k, &value, (value.len() / 2) as u16, 1, &[])
    })
    .collect::<Vec<_>>();

    let table = node("040904b0", &[], 0, 1, &strings); // en-US, Unicode
    let string_info = node("StringFileInfo", &[], 0, 1, &[table]);
    let translation = node("Translation", &[0x09, 0x04, 0xb0, 0x04], 4, 0, &[]);
    let var_info = node("VarFileInfo", &[], 0, 1, &[translation]);
    node("VS_VERSION_INFO", &fixed, fixed.len() as u16, 0, &[string_info, var_info])
}

/// A VERSIONINFO block: length, value length, type, key, value, children.
fn node(key: &str, value: &[u8], value_len: u16, kind: u16, children: &[Vec<u8>]) -> Vec<u8> {
    let mut b = vec![0, 0]; // length, filled in at the end
    b.extend(value_len.to_le_bytes());
    b.extend(kind.to_le_bytes());
    b.extend(utf16z(key));
    pad(&mut b);
    b.extend(value);
    for c in children {
        pad(&mut b);
        b.extend(c);
    }
    let len = b.len() as u16;
    b[..2].copy_from_slice(&len.to_le_bytes());
    b
}

fn utf16z(s: &str) -> Vec<u8> {
    s.encode_utf16().chain([0]).flat_map(u16::to_le_bytes).collect()
}

fn pad(b: &mut Vec<u8>) {
    b.resize(b.len().next_multiple_of(4), 0);
}

fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}

fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
