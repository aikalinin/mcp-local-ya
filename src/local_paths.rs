//! Allowlist for local filesystem paths used by MCP tools.
//!
//! MCP tools such as `yamcli.mail.send` (`attachments`), `yamcli.disk.upload`
//! (`source`), `yamcli.mail.attachment.export` and `yamcli.disk.download`
//! (`output_path`) read or write files on the machine that runs the server.
//! An AI agent driving those tools can be steered by untrusted content (a
//! malicious e-mail, a document on Disk), so the server itself must bound
//! which paths are reachable instead of trusting the caller.
//!
//! Policy:
//! - `YAMCLI_MCP_ALLOWED_PATHS` lists directories (separated by the platform
//!   path-list separator: `:` on Unix, `;` on Windows). Only paths inside one
//!   of them may be read or written by MCP tools.
//! - When the variable is unset or empty, MCP tools get no local filesystem
//!   access at all and return a validation error that explains how to enable it.
//! - Symlinks are resolved before the check, so a link inside an allowed
//!   directory that points outside of it is rejected.
//! - A small deny-list of credential locations (`~/.ssh`, `~/.gnupg`, `~/.aws`,
//!   `~/.kube`, `~/.docker`, `.netrc`, and yamcli's own config directory) is
//!   enforced even inside allowed directories.
//!
//! The CLI is not affected: a human typing `yamcli mail send --attach ./x.pdf`
//! already decided which file to use.

use std::env;
use std::path::{Component, Path, PathBuf};

use crate::error::{Result, YamcliError};
use crate::paths::config_dir;

pub const ALLOWED_PATHS_ENV: &str = "YAMCLI_MCP_ALLOWED_PATHS";

const DENIED_DIR_NAMES: &[&str] = &[".ssh", ".gnupg", ".aws", ".kube", ".docker", ".git"];
const DENIED_FILE_NAMES: &[&str] = &[
    ".netrc",
    "_netrc",
    ".git-credentials",
    ".npmrc",
    ".pypirc",
    ".gitconfig",
    ".bash_history",
    ".zsh_history",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalAccess {
    Read,
    Write,
}

impl LocalAccess {
    fn verb(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

/// Validate a path supplied through an MCP tool argument and return the
/// resolved absolute path that should be used for the actual I/O.
pub fn resolve_mcp_local_path(raw: &str, access: LocalAccess) -> Result<PathBuf> {
    let roots = allowed_roots()?;
    let config = protected_config_dir();
    resolve_against_roots(raw, access, &roots, config.as_deref())
}

/// Same as [`resolve_mcp_local_path`] for a list of paths; fails on the first
/// path outside the allowlist so no partial work is done.
pub fn resolve_mcp_local_paths(raw: &[String], access: LocalAccess) -> Result<Vec<PathBuf>> {
    let roots = allowed_roots()?;
    let config = protected_config_dir();
    raw.iter()
        .map(|path| resolve_against_roots(path, access, &roots, config.as_deref()))
        .collect()
}

/// Human-readable allowlist for status/doctor surfaces. `None` means local
/// filesystem access is disabled for MCP tools.
pub fn configured_allowed_paths() -> Option<Vec<String>> {
    let raw = env::var(ALLOWED_PATHS_ENV).ok()?;
    let entries: Vec<String> = env::split_paths(&raw)
        .map(|path| path.display().to_string())
        .filter(|path| !path.trim().is_empty())
        .collect();
    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

fn allowed_roots() -> Result<Vec<PathBuf>> {
    let Some(raw) = env::var_os(ALLOWED_PATHS_ENV) else {
        return Ok(Vec::new());
    };
    let mut roots = Vec::new();
    for entry in env::split_paths(&raw) {
        let text = entry.to_string_lossy();
        if text.trim().is_empty() {
            continue;
        }
        let expanded = expand_home(&entry);
        let canonical = expanded.canonicalize().map_err(|err| {
            YamcliError::Config(format!(
                "{ALLOWED_PATHS_ENV} entry `{}` cannot be resolved: {err}",
                entry.display()
            ))
        })?;
        if !canonical.is_dir() {
            return Err(YamcliError::Config(format!(
                "{ALLOWED_PATHS_ENV} entry `{}` is not a directory",
                entry.display()
            )));
        }
        reject_overbroad_root(&canonical, &entry)?;
        roots.push(canonical);
    }
    Ok(roots)
}

/// The home directory or a filesystem root as an allowlist entry would hand
/// the agent shell rc files, browser profiles and every credential store; the
/// deny-list below cannot enumerate all of them, so refuse the entry outright.
fn reject_overbroad_root(canonical: &Path, entry: &Path) -> Result<()> {
    let is_fs_root = canonical.parent().is_none();
    let is_home = dirs::home_dir()
        .and_then(|home| home.canonicalize().ok())
        .is_some_and(|home| home == canonical);
    if is_fs_root || is_home {
        return Err(YamcliError::Config(format!(
            "{ALLOWED_PATHS_ENV} entry `{}` is too broad (home directory or filesystem root); \
             list specific working directories instead, e.g. `$HOME/Downloads`",
            entry.display()
        )));
    }
    Ok(())
}

fn protected_config_dir() -> Option<PathBuf> {
    config_dir().ok().and_then(|dir| dir.canonicalize().ok())
}

fn resolve_against_roots(
    raw: &str,
    access: LocalAccess,
    roots: &[PathBuf],
    protected_config_dir: Option<&Path>,
) -> Result<PathBuf> {
    if raw.trim().is_empty() {
        return Err(YamcliError::Validation(
            "local path must not be empty".to_string(),
        ));
    }
    if roots.is_empty() {
        return Err(YamcliError::Validation(format!(
            "MCP tools have no local filesystem access: refusing to {} `{raw}`. \
             Set {ALLOWED_PATHS_ENV} to the directories the MCP server may use \
             (for example `{ALLOWED_PATHS_ENV}=$HOME/Downloads`) and restart the server.",
            access.verb()
        )));
    }

    let requested = expand_home(Path::new(raw));
    let resolved = match access {
        LocalAccess::Read => requested.canonicalize().map_err(|err| {
            YamcliError::Validation(format!("local path `{raw}` cannot be resolved: {err}"))
        })?,
        LocalAccess::Write => resolve_for_write(&requested, raw)?,
    };

    let Some(matched_root) = roots.iter().find(|root| resolved.starts_with(root)) else {
        return Err(YamcliError::Validation(format!(
            "refusing to {} `{raw}`: resolved path `{}` is outside {ALLOWED_PATHS_ENV}",
            access.verb(),
            resolved.display()
        )));
    };
    reject_denied_locations(&resolved, raw, access, protected_config_dir, matched_root)?;
    Ok(resolved)
}

fn resolve_for_write(requested: &Path, raw: &str) -> Result<PathBuf> {
    // The file may not exist yet; the parent must. Resolve the parent through
    // symlinks and re-attach the final component, which must be a plain name.
    let symlink_meta = std::fs::symlink_metadata(requested);
    if let Ok(meta) = &symlink_meta
        && meta.file_type().is_symlink()
    {
        return Err(YamcliError::Validation(format!(
            "refusing to write through symlink `{raw}`"
        )));
    }
    if symlink_meta.is_ok() {
        return requested.canonicalize().map_err(|err| {
            YamcliError::Validation(format!("local path `{raw}` cannot be resolved: {err}"))
        });
    }

    // Walk up to the nearest existing ancestor (tools may create intermediate
    // directories), canonicalize it, and re-attach the missing tail. Every
    // component of the tail must be a plain name: no `..`, no roots, no prefixes.
    let mut existing = requested.to_path_buf();
    let mut tail: Vec<PathBuf> = Vec::new();
    loop {
        match existing.components().next_back() {
            Some(Component::Normal(name)) => tail.push(PathBuf::from(name)),
            Some(Component::CurDir) => {}
            _ => {
                return Err(YamcliError::Validation(format!(
                    "local path `{raw}` must end with a file name"
                )));
            }
        }
        if !existing.pop() {
            existing = PathBuf::from(".");
        }
        if existing.as_os_str().is_empty() {
            existing = PathBuf::from(".");
        }
        if existing.exists() {
            break;
        }
    }
    if tail.is_empty() {
        return Err(YamcliError::Validation(format!(
            "local path `{raw}` must end with a file name"
        )));
    }
    let mut resolved = existing.canonicalize().map_err(|err| {
        YamcliError::Validation(format!(
            "parent directory of `{raw}` cannot be resolved: {err}"
        ))
    })?;
    for component in tail.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn reject_denied_locations(
    resolved: &Path,
    raw: &str,
    access: LocalAccess,
    protected_config_dir: Option<&Path>,
    matched_root: &Path,
) -> Result<()> {
    if let Some(canonical_config) = protected_config_dir
        && resolved.starts_with(canonical_config)
    {
        return Err(YamcliError::Validation(format!(
            "refusing to {} `{raw}`: yamcli configuration directory is never exposed to MCP tools",
            access.verb()
        )));
    }

    // Credential directories are denied wherever they appear in the path.
    for component in resolved.components() {
        let Component::Normal(name) = component else {
            continue;
        };
        let Some(name) = name.to_str() else {
            continue;
        };
        if DENIED_DIR_NAMES.contains(&name) {
            return Err(YamcliError::Validation(format!(
                "refusing to {} `{raw}`: `{name}` holds credentials and is never exposed to MCP tools",
                access.verb()
            )));
        }
    }

    // Below the allowed root, hidden entries (`.git`, `.env`, `.bashrc`, …) are
    // off limits: they are where config, credentials and hooks live, and an
    // agent has no legitimate reason to touch them through mail/disk tools.
    // The root itself may sit under a hidden directory (e.g. ~/.local/share/x).
    if let Ok(relative) = resolved.strip_prefix(matched_root) {
        for component in relative.components() {
            let Component::Normal(name) = component else {
                continue;
            };
            let Some(name) = name.to_str() else {
                continue;
            };
            let trimmed = name.trim_end_matches([' ', '.']);
            if trimmed.starts_with('.') {
                return Err(YamcliError::Validation(format!(
                    "refusing to {} `{raw}`: hidden entry `{name}` is never exposed to MCP tools",
                    access.verb()
                )));
            }
        }
    }

    if let Some(file_name) = resolved.file_name().and_then(|name| name.to_str())
        && DENIED_FILE_NAMES.contains(&file_name)
    {
        return Err(YamcliError::Validation(format!(
            "refusing to {} `{raw}`: `{file_name}` holds credentials and is never exposed to MCP tools",
            access.verb()
        )));
    }
    Ok(())
}

fn expand_home(path: &Path) -> PathBuf {
    let Some(text) = path.to_str() else {
        return path.to_path_buf();
    };
    if text == "~" {
        return dirs::home_dir().unwrap_or_else(|| path.to_path_buf());
    }
    if let Some(rest) = text.strip_prefix("~/").or_else(|| text.strip_prefix("~\\"))
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn roots(dir: &Path) -> Vec<PathBuf> {
        vec![dir.canonicalize().expect("canonical root")]
    }

    #[test]
    fn denies_everything_when_no_roots_are_configured() {
        let temp = tempdir().expect("tempdir");
        let file = temp.path().join("note.txt");
        fs::write(&file, b"x").expect("write");
        let err = resolve_against_roots(file.to_str().unwrap(), LocalAccess::Read, &[], None)
            .expect_err("must be denied");
        assert!(err.to_string().contains(ALLOWED_PATHS_ENV));
    }

    #[test]
    fn allows_read_inside_root_and_rejects_outside() {
        let temp = tempdir().expect("tempdir");
        let allowed = temp.path().join("allowed");
        let other = temp.path().join("other");
        fs::create_dir_all(&allowed).expect("mkdir");
        fs::create_dir_all(&other).expect("mkdir");
        let inside = allowed.join("a.txt");
        let outside = other.join("b.txt");
        fs::write(&inside, b"a").expect("write");
        fs::write(&outside, b"b").expect("write");

        let roots = roots(&allowed);
        assert!(
            resolve_against_roots(inside.to_str().unwrap(), LocalAccess::Read, &roots, None)
                .is_ok()
        );
        let err = resolve_against_roots(outside.to_str().unwrap(), LocalAccess::Read, &roots, None)
            .expect_err("outside must fail");
        assert!(err.to_string().contains("outside"));
    }

    #[test]
    fn rejects_parent_traversal_out_of_root() {
        let temp = tempdir().expect("tempdir");
        let allowed = temp.path().join("allowed");
        fs::create_dir_all(&allowed).expect("mkdir");
        let secret = temp.path().join("secret.txt");
        fs::write(&secret, b"s").expect("write");

        let sneaky = allowed.join("..").join("secret.txt");
        let err = resolve_against_roots(
            sneaky.to_str().unwrap(),
            LocalAccess::Read,
            &roots(&allowed),
            None,
        )
        .expect_err("traversal must fail");
        assert!(err.to_string().contains("outside"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escaping_root() {
        let temp = tempdir().expect("tempdir");
        let allowed = temp.path().join("allowed");
        fs::create_dir_all(&allowed).expect("mkdir");
        let secret = temp.path().join("secret.txt");
        fs::write(&secret, b"s").expect("write");
        let link = allowed.join("link.txt");
        std::os::unix::fs::symlink(&secret, &link).expect("symlink");

        let err = resolve_against_roots(
            link.to_str().unwrap(),
            LocalAccess::Read,
            &roots(&allowed),
            None,
        )
        .expect_err("symlink escape must fail");
        assert!(err.to_string().contains("outside"));

        let err = resolve_against_roots(
            link.to_str().unwrap(),
            LocalAccess::Write,
            &roots(&allowed),
            None,
        )
        .expect_err("writing through symlink must fail");
        assert!(err.to_string().contains("symlink"));
    }

    #[test]
    fn write_target_may_not_exist_yet_and_may_create_subdirectories() {
        let temp = tempdir().expect("tempdir");
        let allowed = temp.path().join("allowed");
        fs::create_dir_all(&allowed).expect("mkdir");
        let canonical_allowed = allowed.canonicalize().expect("canonical");

        let fresh = allowed.join("new.pdf");
        let resolved = resolve_against_roots(
            fresh.to_str().unwrap(),
            LocalAccess::Write,
            &roots(&allowed),
            None,
        )
        .expect("new file inside root");
        assert_eq!(resolved, canonical_allowed.join("new.pdf"));

        let nested = allowed.join("downloads").join("new.pdf");
        let resolved = resolve_against_roots(
            nested.to_str().unwrap(),
            LocalAccess::Write,
            &roots(&allowed),
            None,
        )
        .expect("nested file inside root");
        assert_eq!(
            resolved,
            canonical_allowed.join("downloads").join("new.pdf")
        );

        // `..` inside the not-yet-existing tail cannot be used to climb out.
        let sneaky = allowed
            .join("downloads")
            .join("..")
            .join("..")
            .join("x.pdf");
        assert!(
            resolve_against_roots(
                sneaky.to_str().unwrap(),
                LocalAccess::Write,
                &roots(&allowed),
                None
            )
            .is_err()
        );
    }

    #[test]
    fn denies_credential_directories_even_inside_root() {
        let temp = tempdir().expect("tempdir");
        let ssh = temp.path().join(".ssh");
        fs::create_dir_all(&ssh).expect("mkdir");
        let key = ssh.join("id_ed25519");
        fs::write(&key, b"k").expect("write");
        let netrc = temp.path().join(".netrc");
        fs::write(&netrc, b"n").expect("write");

        let roots = roots(temp.path());
        let err = resolve_against_roots(key.to_str().unwrap(), LocalAccess::Read, &roots, None)
            .expect_err(".ssh must be denied");
        assert!(err.to_string().contains(".ssh"));
        let err = resolve_against_roots(netrc.to_str().unwrap(), LocalAccess::Read, &roots, None)
            .expect_err(".netrc must be denied");
        assert!(err.to_string().contains(".netrc"));
    }

    #[test]
    fn denies_yamcli_config_dir_even_inside_root() {
        let temp = tempdir().expect("tempdir");
        let config = temp.path().join("yamcli-config");
        fs::create_dir_all(&config).expect("mkdir");
        let creds = config.join("credentials.toml");
        fs::write(&creds, b"secret").expect("write");
        let canonical_config = config.canonicalize().expect("canonical config");

        let err = resolve_against_roots(
            creds.to_str().unwrap(),
            LocalAccess::Read,
            &roots(temp.path()),
            Some(&canonical_config),
        )
        .expect_err("config dir must be denied");
        assert!(err.to_string().contains("configuration directory"));
    }
}

#[cfg(test)]
mod hardening_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn roots(dir: &Path) -> Vec<PathBuf> {
        vec![dir.canonicalize().expect("canonical root")]
    }

    #[test]
    fn denies_hidden_entries_below_root_for_read_and_write() {
        let temp = tempdir().expect("tempdir");
        let git = temp.path().join("repo").join(".git");
        fs::create_dir_all(&git).expect("mkdir");
        let git_config = git.join("config");
        fs::write(&git_config, b"[core]").expect("write");
        let env_file = temp.path().join("repo").join(".env");
        fs::write(&env_file, b"SECRET=1").expect("write");
        let roots = roots(temp.path());

        for path in [&git_config, &env_file] {
            let err =
                resolve_against_roots(path.to_str().unwrap(), LocalAccess::Read, &roots, None)
                    .expect_err("hidden read must be denied");
            assert!(
                err.to_string().contains("hidden") || err.to_string().contains(".git"),
                "{err}"
            );
        }

        // Writing a not-yet-existing hidden file (shell rc, hook) is denied too,
        // including the Windows trailing-space/dot spelling.
        for name in [".bashrc", ".ssh /authorized_keys", ".git./hooks"] {
            let target = temp.path().join("repo").join(name);
            let err =
                resolve_against_roots(target.to_str().unwrap(), LocalAccess::Write, &roots, None)
                    .expect_err("hidden write must be denied");
            assert!(
                err.to_string().contains("hidden") || err.to_string().contains("credentials"),
                "{name}: {err}"
            );
        }

        // A visible file next to them is fine.
        let ok = temp.path().join("repo").join("README.md");
        fs::write(&ok, b"# hi").expect("write");
        assert!(
            resolve_against_roots(ok.to_str().unwrap(), LocalAccess::Read, &roots, None).is_ok()
        );
    }

    #[test]
    fn root_itself_may_live_under_a_hidden_directory() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join(".local").join("share").join("exchange");
        fs::create_dir_all(&root).expect("mkdir");
        let file = root.join("report.pdf");
        fs::write(&file, b"%PDF").expect("write");
        assert!(
            resolve_against_roots(
                file.to_str().unwrap(),
                LocalAccess::Read,
                &roots(&root),
                None
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_home_and_filesystem_root_as_allowlist_entries() {
        let fs_root = if cfg!(windows) {
            PathBuf::from("C:\\")
        } else {
            PathBuf::from("/")
        };
        let err = reject_overbroad_root(&fs_root.canonicalize().expect("root"), &fs_root)
            .expect_err("filesystem root must be rejected");
        assert!(err.to_string().contains("too broad"), "{err}");

        if let Some(home) = dirs::home_dir().and_then(|home| home.canonicalize().ok()) {
            let err = reject_overbroad_root(&home, &home).expect_err("home must be rejected");
            assert!(err.to_string().contains("too broad"), "{err}");
        }

        let temp = tempdir().expect("tempdir");
        let canonical = temp.path().canonicalize().expect("canonical");
        assert!(reject_overbroad_root(&canonical, temp.path()).is_ok());
    }
}
