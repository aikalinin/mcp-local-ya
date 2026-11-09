//! Helpers shared by unit tests.
//!
//! `YAMCLI_CONFIG_DIR` and `YAMCLI_SECRET_BACKEND` are process-wide, while unit
//! tests run in parallel threads. Every test that sets these variables, or calls
//! code that reads the config directory, must hold [`lock_config_env`] for its
//! whole duration. Otherwise one test can load (and, with the keyring backend,
//! migrate and delete) another test's credentials file.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use tempfile::TempDir;

static CONFIG_ENV_LOCK: Mutex<()> = Mutex::new(());

const CONFIG_DIR_ENV: &str = "YAMCLI_CONFIG_DIR";
const SECRET_BACKEND_ENV: &str = "YAMCLI_SECRET_BACKEND";

/// Serialises access to the config-related environment variables.
///
/// A panicking test poisons the mutex; the guard is recovered so one failure
/// does not cascade into `PoisonError` failures in unrelated tests.
pub(crate) fn lock_config_env() -> MutexGuard<'static, ()> {
    CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Points the config directory at a fresh temporary directory with the file
/// secret backend, so the test never touches the developer's real config.
/// Holds the config env lock until dropped and then clears both variables.
pub(crate) struct IsolatedConfigDir {
    dir: TempDir,
    _guard: MutexGuard<'static, ()>,
}

impl IsolatedConfigDir {
    pub(crate) fn new() -> Self {
        let guard = lock_config_env();
        let dir = tempfile::tempdir().expect("isolated config tempdir");
        // SAFETY: every reader and writer of these variables in unit tests
        // holds CONFIG_ENV_LOCK, which `guard` keeps locked.
        unsafe {
            std::env::set_var(CONFIG_DIR_ENV, dir.path());
            std::env::set_var(SECRET_BACKEND_ENV, "file");
        }
        Self { dir, _guard: guard }
    }

    #[allow(dead_code)]
    pub(crate) fn path(&self) -> &Path {
        self.dir.path()
    }
}

impl Drop for IsolatedConfigDir {
    fn drop(&mut self) {
        // SAFETY: the config env lock is still held; `_guard` drops after this.
        unsafe {
            std::env::remove_var(CONFIG_DIR_ENV);
            std::env::remove_var(SECRET_BACKEND_ENV);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_config_dir_sets_and_clears_env() {
        let path = {
            let isolated = IsolatedConfigDir::new();
            assert_eq!(
                std::env::var_os(CONFIG_DIR_ENV).as_deref(),
                Some(isolated.path().as_os_str())
            );
            assert_eq!(std::env::var(SECRET_BACKEND_ENV).as_deref(), Ok("file"));
            isolated.path().to_path_buf()
        };

        let _guard = lock_config_env();
        assert!(std::env::var_os(CONFIG_DIR_ENV).is_none());
        assert!(std::env::var_os(SECRET_BACKEND_ENV).is_none());
        assert!(!path.exists());
    }

    #[test]
    fn lock_recovers_after_a_panicking_holder() {
        let result = std::thread::spawn(|| {
            let _guard = lock_config_env();
            panic!("simulated test failure while holding the lock");
        })
        .join();
        assert!(result.is_err());

        // Must not propagate PoisonError to the next test.
        let _guard = lock_config_env();
    }
}
