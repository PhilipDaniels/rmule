use crate::times;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

/// Check to see if the file exists, and really is a file.
pub fn file_exists(file: &Path) -> Result<bool> {
    if file.try_exists()? {
        if file.is_file() {
            Ok(true)
        } else {
            bail!("{} is not a file", file.to_string_lossy())
        }
    } else {
        Ok(false)
    }
}

/// Check to see if the directory exists, and really is a directory.
pub fn directory_exists(dir: &Path) -> Result<bool> {
    if dir.try_exists()? {
        if dir.is_dir() {
            Ok(true)
        } else {
            bail!("{} is not a directory", dir.to_string_lossy())
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
            bail!("{} is not a directory", dir.to_string_lossy())
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

/// Constructs a backup filename for an existing file by appending a
/// date-time string to the filename. This function will panic if
/// passed something without a terminal filename.
pub fn make_backup_filename<P: Into<PathBuf>>(path: P) -> PathBuf {
    let original = path.into();

    let new_file_name = format!(
        "{}-{}",
        original.file_name().unwrap().to_string_lossy(),
        times::now_to_yyyy_mm_dd_hh_mm_ss()
    );

    original.with_file_name(new_file_name)
}

/// Delete backups of files in 'directory' that begin with 'filename' and
/// have our known date backup suffix. 'num_to_keep' specifies how many
/// backups to retain; it can be zero.
///
/// Returns the number of files that were deleted.
pub fn delete_backups<P, Q>(directory: P, filename: Q, num_to_keep: usize) -> Result<usize>
where
    P: Into<PathBuf>,
    Q: Into<PathBuf>,
{
    let directory = directory.into();
    let filename: String = filename.into().to_string_lossy().into();
    let mut num_deleted = 0;

    let mut backups_to_delete = Vec::new();

    for entry in directory.read_dir()? {
        let path = entry?.path();
        if path.is_file() {
            if let Some(fname) = path.file_name() {
                let fname: String = fname.to_string_lossy().into();
                if fname.starts_with(&filename) && has_backup_suffix(&fname) {
                    backups_to_delete.push(path);
                }
            }
        }
    }

    backups_to_delete.sort();

    for backup in backups_to_delete.into_iter().rev().skip(num_to_keep) {
        std::fs::remove_file(backup)?;
        num_deleted += 1;
    }

    Ok(num_deleted)
}

/// Check for suffix of -YYYY-MM-DDTHH-MM-Ss. No need to bring in regex crate
/// for this.
fn has_backup_suffix(filename: &str) -> bool {
    const SUFFIX_LEN: usize = "-YYYY-MM-DDTHH-MM-SS".len();
    if filename.len() < SUFFIX_LEN {
        return false;
    }

    let suffix: Vec<_> = filename[filename.len() - SUFFIX_LEN..].chars().collect();

    suffix[0] == '-'
        && suffix[1].is_ascii_digit()
        && suffix[2].is_ascii_digit()
        && suffix[3].is_ascii_digit()
        && suffix[4].is_ascii_digit()
        && suffix[5] == '-'
        && suffix[6].is_ascii_digit()
        && suffix[7].is_ascii_digit()
        && suffix[8] == '-'
        && suffix[9].is_ascii_digit()
        && suffix[10].is_ascii_digit()
        && suffix[11] == 'T'
        && suffix[12].is_ascii_digit()
        && suffix[13].is_ascii_digit()
        && suffix[14] == '-'
        && suffix[15].is_ascii_digit()
        && suffix[16].is_ascii_digit()
        && suffix[17] == '-'
        && suffix[18].is_ascii_digit()
        && suffix[19].is_ascii_digit()
}

/// Makes a guaranteed-absolute path from a filename that may or may not
/// already be absolute. Returns a Cow::Borrowed if filename is already
/// absolute, else returns a Cow::Owned.
pub fn make_absolute<'a, 'b>(filename: &'a Path, directory: &'b Path) -> Cow<'a, Path> {
    if filename.is_absolute() {
        filename.into()
    } else {
        let mut d = directory.to_owned();
        d.push(filename);
        d.into()
    }
}
