use std::path::Path;

use anyhow::{Result, bail};

/// Check to see if the directory exists, and really is a directory.
pub fn directory_exists(config_dir: &Path) -> Result<bool> {
    if config_dir.try_exists()? {
        if config_dir.is_dir() {
            Ok(true)
        } else {
            bail!("{} is not a directory", config_dir.to_string_lossy())
        }
    } else {
        Ok(false)
    }
}

/// Ensures that a directory exists by creating it and all its parents
/// if necessary, and making it writable. We also check that the passed
/// path IS actually a directory and not a symlink or a file etc.
pub fn ensure_directory_exists(dir: &Path) -> Result<()> {
    if dir.try_exists()? {
        if !dir.is_dir() {
            bail!("The path {} is not a directory", dir.to_string_lossy());
        }
    } else {
        std::fs::create_dir_all(dir)?;
    }

    ensure_writable(dir)
}

/// Ensures that a path is writable. Path can be a directory or a file.
pub fn ensure_writable(path: &Path) -> Result<()> {
    let mut perms = std::fs::metadata(path)?.permissions();
    if perms.readonly() {
        perms.set_readonly(false);
        std::fs::set_permissions(path, perms)?;
    }

    Ok(())
}