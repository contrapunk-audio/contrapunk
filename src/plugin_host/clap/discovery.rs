//! Filesystem discovery of CLAP plugins.
//!
//! Walks the OS-standard CLAP directories, plus any paths listed in
//! the `CLAP_PATH` environment variable, and returns every `.clap`
//! bundle found. Each bundle is described by a [`PluginDescriptor`].
//!
//! This module intentionally does **not** load the plugin entry point
//! to extract the full descriptor (id/name/vendor/version). That
//! requires dynamically loading the `.clap` binary, which can have
//! side effects (static initialisers, audio thread creation by
//! misbehaving plugins) and is not safe to do during app startup. For
//! v1 we return the *file-level* descriptor — the bundle path + a
//! best-guess id/name derived from the filename. Fetching the real
//! manifest is deferred to actual plugin instantiation.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Metadata describing a discovered CLAP plugin bundle.
///
/// Serialised to the UI via Tauri. When real manifest introspection
/// lands, `id`/`name`/`vendor`/`version` will be populated from the
/// plugin's own `clap_plugin_descriptor`. For now `id` and `name` are
/// derived from the filename and the other fields are blank strings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDescriptor {
    /// Stable plugin identifier. For file-level discovery we use the
    /// bundle's path; after instantiation this will be replaced by the
    /// plugin's own `id` (e.g. `com.surge-synth-team.surge-xt`).
    pub id: String,

    /// Human-readable display name. Derived from the filename at
    /// discovery time; replaced by the plugin's `name` on load.
    pub name: String,

    /// Plugin vendor. Blank until we load the manifest.
    pub vendor: String,

    /// Plugin version. Blank until we load the manifest.
    pub version: String,

    /// Absolute path to the `.clap` bundle on disk.
    pub path: PathBuf,
}

/// Scan every standard CLAP directory for this OS and return the list
/// of discovered plugins.
///
/// The scan is best-effort: unreadable directories are silently
/// skipped so one bad entry in `CLAP_PATH` does not nuke discovery.
/// Duplicates (same path discovered via multiple roots) are
/// de-duplicated.
pub fn discover_plugins() -> Vec<PluginDescriptor> {
    let roots = standard_clap_paths();
    discover_in(&roots)
}

/// Discover plugins under the given list of root directories. Exposed
/// for tests that need to point at a tempdir.
pub fn discover_in(roots: &[PathBuf]) -> Vec<PluginDescriptor> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        walk_for_clap(root, &mut |path| {
            if seen.insert(path.to_path_buf()) {
                out.push(descriptor_from_path(path));
            }
        });
    }
    out
}

/// Return the OS-appropriate list of root directories where CLAP
/// plugins live. Follows the spec at
/// <https://github.com/free-audio/clap/blob/main/include/clap/entry.h>.
///
/// * macOS: `~/Library/Audio/Plug-Ins/CLAP/`,
///   `/Library/Audio/Plug-Ins/CLAP/`
/// * Linux: `~/.clap/`, `/usr/lib/clap/`, `/usr/local/lib/clap/`
/// * Windows: `%COMMONPROGRAMFILES%\CLAP\`,
///   `%LOCALAPPDATA%\Programs\Common\CLAP\`
///
/// In addition, every entry of `$CLAP_PATH` (platform path separator)
/// is appended.
pub fn standard_clap_paths() -> Vec<PathBuf> {
    let mut paths = platform_paths();
    for extra in env_clap_paths() {
        paths.push(extra);
    }
    paths
}

#[cfg(target_os = "macos")]
fn platform_paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(home) = home_dir() {
        v.push(home.join("Library/Audio/Plug-Ins/CLAP"));
    }
    v.push(PathBuf::from("/Library/Audio/Plug-Ins/CLAP"));
    v
}

#[cfg(target_os = "linux")]
fn platform_paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(home) = home_dir() {
        v.push(home.join(".clap"));
    }
    v.push(PathBuf::from("/usr/lib/clap"));
    v.push(PathBuf::from("/usr/local/lib/clap"));
    v
}

#[cfg(target_os = "windows")]
fn platform_paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(common) = std::env::var("COMMONPROGRAMFILES") {
        v.push(PathBuf::from(common).join("CLAP"));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        v.push(
            PathBuf::from(local)
                .join("Programs")
                .join("Common")
                .join("CLAP"),
        );
    }
    v
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_paths() -> Vec<PathBuf> {
    Vec::new()
}

fn home_dir() -> Option<PathBuf> {
    // std::env::home_dir is deprecated and quirky; roll our own via
    // $HOME (unix) / %USERPROFILE% (win).
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    if let Ok(h) = std::env::var("USERPROFILE") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    None
}

fn env_clap_paths() -> Vec<PathBuf> {
    let Ok(raw) = std::env::var("CLAP_PATH") else {
        return Vec::new();
    };
    let sep = if cfg!(windows) { ';' } else { ':' };
    raw.split(sep)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Recursively walk `root` looking for `.clap` bundles. Calls `f` with
/// the absolute path of every bundle found. Silently skips entries the
/// process cannot read. Bounded recursion depth prevents runaway
/// symlink cycles on weirdly-configured systems.
fn walk_for_clap(root: &Path, f: &mut dyn FnMut(&Path)) {
    walk_for_clap_bounded(root, f, 0);
}

const MAX_DEPTH: u32 = 8;

fn walk_for_clap_bounded(dir: &Path, f: &mut dyn FnMut(&Path), depth: u32) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_clap = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("clap"))
            .unwrap_or(false);
        if is_clap {
            // On macOS a `.clap` is a bundle (directory). On Linux and
            // Windows it's a shared library file. We report either.
            f(&path);
            continue;
        }
        // Only descend into real directories (not `.clap` bundles,
        // which we've already reported above).
        if let Ok(ft) = entry.file_type() {
            if ft.is_dir() && !ft.is_symlink() {
                walk_for_clap_bounded(&path, f, depth + 1);
            }
        }
    }
}

fn descriptor_from_path(path: &Path) -> PluginDescriptor {
    let filename_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    PluginDescriptor {
        id: path.to_string_lossy().into_owned(),
        name: filename_stem,
        vendor: String::new(),
        version: String::new(),
        path: path.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Build a temp directory containing a fake `.clap` file and a
    /// nested structure; verify discovery picks it up.
    #[test]
    fn discovers_clap_file_in_tempdir() {
        let tmp =
            std::env::temp_dir().join(format!("contrapunk-clap-disco-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("nested/deep")).unwrap();
        let plugin_path = tmp.join("nested/deep/FakeSynth.clap");
        fs::write(&plugin_path, b"not really a plugin").unwrap();

        let descriptors = discover_in(&[tmp.clone()]);

        let found = descriptors.iter().find(|d| d.path == plugin_path);
        assert!(
            found.is_some(),
            "expected to find FakeSynth.clap; got {descriptors:?}"
        );
        let d = found.unwrap();
        assert_eq!(d.name, "FakeSynth");
        assert!(!d.id.is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn missing_root_is_silently_skipped() {
        let nonsense = PathBuf::from("/definitely/not/a/real/path/contrapunk-xyz-123");
        let out = discover_in(&[nonsense]);
        assert!(out.is_empty());
    }

    #[test]
    fn duplicate_paths_deduplicated() {
        let tmp = std::env::temp_dir().join(format!("contrapunk-clap-dup-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let plugin_path = tmp.join("Dup.clap");
        fs::write(&plugin_path, b"x").unwrap();

        // Pass the same root twice.
        let out = discover_in(&[tmp.clone(), tmp.clone()]);
        let matches = out.iter().filter(|d| d.path == plugin_path).count();
        assert_eq!(matches, 1, "duplicates should be de-duplicated");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn env_clap_path_is_parsed() {
        // Set a known CLAP_PATH, read it back, restore.
        let prev = std::env::var("CLAP_PATH").ok();
        let sep = if cfg!(windows) { ";" } else { ":" };
        let value = format!("/tmp/a{sep}/tmp/b");
        std::env::set_var("CLAP_PATH", &value);

        let parsed = env_clap_paths();
        assert_eq!(
            parsed,
            vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")]
        );

        match prev {
            Some(v) => std::env::set_var("CLAP_PATH", v),
            None => std::env::remove_var("CLAP_PATH"),
        }
    }

    #[test]
    fn standard_paths_are_platform_appropriate() {
        let paths = platform_paths();
        // We can't assert on absolute contents (CI may not have HOME),
        // but we can assert that the function returns something or
        // nothing deterministically.
        if cfg!(target_os = "macos") {
            assert!(paths
                .iter()
                .any(|p| p.ends_with("Library/Audio/Plug-Ins/CLAP")));
        } else if cfg!(target_os = "linux") {
            assert!(paths.iter().any(|p| p == &PathBuf::from("/usr/lib/clap")));
        } else if cfg!(target_os = "windows") {
            // Windows paths depend entirely on env vars; just sanity-
            // check that if COMMONPROGRAMFILES is set, the path ends
            // with CLAP.
            if std::env::var("COMMONPROGRAMFILES").is_ok() {
                assert!(paths.iter().any(|p| p.ends_with("CLAP")));
            }
        }
    }

    #[test]
    fn descriptor_from_path_uses_file_stem() {
        let d = descriptor_from_path(Path::new("/foo/bar/SuperSynth.clap"));
        assert_eq!(d.name, "SuperSynth");
        assert_eq!(d.path, PathBuf::from("/foo/bar/SuperSynth.clap"));
        assert!(d.vendor.is_empty());
    }
}
