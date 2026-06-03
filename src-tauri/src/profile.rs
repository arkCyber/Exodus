//! Exodus Browser — profile directories (default vs private/incognito storage).

use std::path::{Path, PathBuf};

/// Profile folder name for normal browsing.
pub const PROFILE_DEFAULT: &str = "default";

/// Profile folder name for private / incognito browsing.
pub const PROFILE_PRIVATE: &str = "private";

/// Return `app_data/profiles/{default|private}`.
pub fn profile_dir(app_data: &Path, private_mode: bool) -> PathBuf {
    let name = if private_mode {
        PROFILE_PRIVATE
    } else {
        PROFILE_DEFAULT
    };
    app_data.join("profiles").join(name)
}

/// History JSON store for the active profile.
pub fn profile_history_dir(app_data: &Path, private_mode: bool) -> PathBuf {
    profile_dir(app_data, private_mode).join("history")
}

/// Cookie jar JSON store for the active profile.
pub fn profile_cookie_dir(app_data: &Path, private_mode: bool) -> PathBuf {
    profile_dir(app_data, private_mode).join("cookies")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_dirs_differ_for_private() {
        let root = PathBuf::from("/tmp/exodus-test");
        let normal = profile_dir(&root, false);
        let private = profile_dir(&root, true);
        assert!(normal.ends_with("profiles/default"));
        assert!(private.ends_with("profiles/private"));
        assert_ne!(normal, private);
    }
}
