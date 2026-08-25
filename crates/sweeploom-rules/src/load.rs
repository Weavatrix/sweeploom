//! Load TOML rule packs from disk. Never execute them.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{RulePack, parse_pack};

const MAX_DEPTH: u32 = 4;

/// One TOML file from a rules directory.
pub struct LoadedRuleFile {
    /// Path of the file that was read.
    pub path: PathBuf,
    /// Parsed pack, or a parse/IO message. Never executed.
    pub pack: Result<RulePack, String>,
}

/// Read `*.toml` under `root`. Missing directory is empty, not an error.
pub fn load_packs(root: &Path) -> io::Result<Vec<LoadedRuleFile>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    walk(root, 0, &mut out)?;
    out.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(out)
}

fn walk(dir: &Path, depth: u32, out: &mut Vec<LoadedRuleFile>) -> io::Result<()> {
    if depth > MAX_DEPTH {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            walk(&path, depth + 1, out)?;
        } else if path.extension().and_then(|item| item.to_str()) == Some("toml") {
            let pack = match fs::read_to_string(&path) {
                Ok(text) => parse_pack(&text).map_err(|error| error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            out.push(LoadedRuleFile { path, pack });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_dir_is_empty() {
        let path = PathBuf::from("definitely-missing-sweeploom-rules-dir");
        assert!(load_packs(&path).unwrap().is_empty());
    }

    #[test]
    fn loads_vite_cache_pack() {
        let dir = std::env::temp_dir().join(format!("sweeploom-rules-{}", std::process::id()));
        let nested = dir.join("common");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            nested.join("vite-cache.toml"),
            r#"
schema = 1
[[cleaner]]
id = "vite-cache"
label = "Vite cache"
risk = "safe"
strategy = "permanent-generated"
"#,
        )
        .unwrap();
        let files = load_packs(&dir).unwrap();
        let pack = files[0].pack.as_ref().expect("parse");
        assert_eq!(pack.cleaner[0].id, "vite-cache");
        let _ = fs::remove_dir_all(&dir);
    }
}
