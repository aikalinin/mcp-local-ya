use std::path::{Path, PathBuf};

use crate::error::{Result, YamcliError};

const CONFIG_DIR_NAME: &str = "yamcli";
/// Directory name used before the project was renamed from `yacli` to `yamcli`.
const LEGACY_CONFIG_DIR_NAME: &str = "yacli";

pub fn config_dir() -> Result<PathBuf> {
    if let Some(value) = std::env::var_os("YAMCLI_CONFIG_DIR") {
        return Ok(PathBuf::from(value));
    }

    let base = dirs::config_dir()
        .ok_or_else(|| YamcliError::Config("Could not resolve config directory".to_string()))?;
    Ok(default_config_dir_in(&base))
}

/// Returns `<base>/yamcli`, moving a legacy `<base>/yacli` directory into place
/// on first use so existing accounts, credentials and activity survive the rename.
/// A failed move is not fatal: the caller simply starts with a fresh directory.
fn default_config_dir_in(base: &Path) -> PathBuf {
    let current = base.join(CONFIG_DIR_NAME);
    if !current.exists() {
        let legacy = base.join(LEGACY_CONFIG_DIR_NAME);
        if legacy.is_dir() {
            let _ = std::fs::rename(&legacy, &current);
        }
    }
    current
}

pub fn accounts_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("accounts.toml"))
}

pub fn credentials_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("credentials.toml"))
}

pub fn activity_log_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("activity.toml"))
}

pub fn oauth_sessions_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("oauth_sessions.toml"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn default_config_dir_moves_legacy_yacli_directory() {
        let base = tempdir().expect("tempdir");
        let legacy = base.path().join("yacli");
        fs::create_dir_all(&legacy).expect("legacy dir");
        fs::write(legacy.join("accounts.toml"), "version = 1\n").expect("legacy file");

        let dir = default_config_dir_in(base.path());

        assert_eq!(dir, base.path().join("yamcli"));
        assert!(!legacy.exists());
        assert_eq!(
            fs::read_to_string(dir.join("accounts.toml")).expect("migrated file"),
            "version = 1\n"
        );
    }

    #[test]
    fn default_config_dir_keeps_existing_directory_and_legacy_untouched() {
        let base = tempdir().expect("tempdir");
        fs::create_dir_all(base.path().join("yamcli")).expect("current dir");
        fs::create_dir_all(base.path().join("yacli")).expect("legacy dir");

        let dir = default_config_dir_in(base.path());

        assert_eq!(dir, base.path().join("yamcli"));
        assert!(base.path().join("yacli").exists());
    }

    #[test]
    fn default_config_dir_without_legacy_does_not_create_anything() {
        let base = tempdir().expect("tempdir");

        let dir = default_config_dir_in(base.path());

        assert_eq!(dir, base.path().join("yamcli"));
        assert!(!dir.exists());
    }
}
