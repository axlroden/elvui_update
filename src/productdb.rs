//! Minimal reader for Battle.net's product.db (a protobuf message).
//!
//! Rather than depending on generated protobuf code, which breaks when
//! Blizzard adds fields, we read only the fields we need:
//!
//!   Database.productInstall    = 1 (repeated)
//!   ProductInstall.productCode = 2
//!   ProductInstall.settings    = 3
//!   UserSettings.installPath   = 1
//!   UserSettings.productExtra  = 13

/// One WoW install listed in product.db.
#[derive(Debug, Clone)]
pub struct Install {
    /// e.g. wow, wow_classic, wow_classic_era, wow_classic_beta
    pub code: String,
    /// e.g. C:/Program Files (x86)/World of Warcraft
    pub path: String,
    /// e.g. _retail_, _classic_
    pub extra: String,
}

pub fn read(path: &std::path::Path) -> Result<Vec<Install>, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    parse(&data)
}

pub fn parse(data: &[u8]) -> Result<Vec<Install>, String> {
    let mut out = Vec::new();
    for (field, val) in fields(data)? {
        let Some(install) = val.filter(|_| field == 1) else {
            continue;
        };
        let code = text(first(install, 2));
        if !code.starts_with("wow") {
            continue;
        }
        let settings = first(install, 3).unwrap_or(&[]);
        out.push(Install {
            code,
            path: text(first(settings, 1)),
            extra: text(first(settings, 13)),
        });
    }
    Ok(out)
}

const CORRUPT: &str = "product.db: truncated or corrupt data";

fn varint(b: &[u8], i: &mut usize) -> Result<u64, String> {
    let mut r = 0u64;
    for shift in (0..64).step_by(7) {
        let c = *b.get(*i).ok_or(CORRUPT)?;
        *i += 1;
        r |= u64::from(c & 0x7f) << shift;
        if c < 0x80 {
            return Ok(r);
        }
    }
    Err(CORRUPT.into())
}

/// (field number, bytes of a length-delimited value)
type Field<'a> = (u64, Option<&'a [u8]>);

/// All fields of a message; the value is Some for length-delimited fields.
fn fields(b: &[u8]) -> Result<Vec<Field<'_>>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let key = varint(b, &mut i)?;
        let mut val = None;
        match key & 7 {
            0 => {
                varint(b, &mut i)?;
            }
            1 => i += 8,
            2 => {
                let len = varint(b, &mut i)? as usize;
                let end = i
                    .checked_add(len)
                    .filter(|&e| e <= b.len())
                    .ok_or(CORRUPT)?;
                val = Some(&b[i..end]);
                i = end;
            }
            5 => i += 4,
            _ => return Err(CORRUPT.into()),
        }
        if i > b.len() {
            return Err(CORRUPT.into());
        }
        out.push((key >> 3, val));
    }
    Ok(out)
}

fn first(b: &[u8], want: u64) -> Option<&[u8]> {
    fields(b)
        .ok()?
        .into_iter()
        .find(|(f, v)| *f == want && v.is_some())?
        .1
}

fn text(b: Option<&[u8]>) -> String {
    String::from_utf8_lossy(b.unwrap_or(&[])).into_owned()
}
