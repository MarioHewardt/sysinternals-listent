//! Filesystem trust checks for the root LaunchDaemon, not standalone daemon runs.

use anyhow::{bail, Context, Result};
use std::fs::{self, DirBuilder, Metadata, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

fn check_owner_and_mode(uid: u32, mode: u32, path: &Path) -> Result<()> {
    if uid != 0 || mode & 0o022 != 0 {
        bail!(
            "Permission denied: daemon resource must be root-owned and not group/other-writable: {}",
            path.display()
        );
    }
    Ok(())
}

// ACLs can grant writes independently of POSIX mode bits. Accept deny-only ACLs
// (including the standard /Library ACL); reject allow entries conservatively.
#[cfg(target_os = "macos")]
fn check_acl(path: &Path) -> Result<()> {
    let output = std::process::Command::new("/bin/ls")
        .env("LC_ALL", "C")
        // Numeric output also renders ACL identities as UUIDs, not names with spaces.
        .args(["-lden", "--"])
        .arg(path)
        .output()
        .context("Failed to inspect daemon resource ACL")?;
    if !output.status.success() {
        bail!(
            "Failed to inspect ACL for {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let listing = std::str::from_utf8(&output.stdout)
        .context("Invalid UTF-8 in daemon resource ACL listing")?;
    check_acl_listing(listing, path)
}

#[cfg(any(target_os = "macos", test))]
fn check_acl_listing(listing: &str, path: &Path) -> Result<()> {
    let mut lines = listing.lines();
    let header = lines
        .next()
        .context("Missing daemon resource ACL listing")?;
    let mode = header
        .split_whitespace()
        .next()
        .context("Missing file mode")?;
    let mut entries = 0;
    for entry in lines {
        let fields: Vec<_> = entry.split_whitespace().collect();
        let deny_entry = matches!(
            fields.as_slice(),
            [_, _, "deny", _] | [_, _, "inherited", "deny", _]
        );
        if !deny_entry
            || fields[0]
                .strip_suffix(':')
                .and_then(|n| n.parse::<usize>().ok())
                != Some(entries)
        {
            bail!(
                "Permission denied: daemon resource has an unsupported or allow ACL: {}",
                path.display()
            );
        }
        entries += 1;
    }
    if mode.contains('+') && entries == 0 {
        bail!(
            "Incomplete ACL listing for daemon resource: {}",
            path.display()
        );
    }
    Ok(())
}

fn check_metadata(metadata: &Metadata, path: &Path) -> Result<()> {
    check_owner_and_mode(metadata.uid(), metadata.mode(), path)?;
    #[cfg(target_os = "macos")]
    check_acl(path)?;
    Ok(())
}

fn check_path_text(path: &Path) -> Result<()> {
    let text = path
        .to_str()
        .context("Daemon resource path contains invalid UTF-8")?;
    if text.chars().any(char::is_control) {
        bail!("Daemon resource path contains control characters");
    }
    Ok(())
}

/// Resolve aliases, then verify the target and every ancestor. Callers must use
/// the returned path so a replaceable alias cannot redirect the installed job.
pub fn protected_path(path: &Path, directory: bool) -> Result<PathBuf> {
    let resolved = fs::canonicalize(path)
        .with_context(|| format!("Failed to resolve daemon resource: {}", path.display()))?;
    check_path_text(&resolved)?;
    for (index, component) in resolved.ancestors().enumerate() {
        let metadata = fs::symlink_metadata(component).with_context(|| {
            format!("Failed to inspect daemon resource: {}", component.display())
        })?;
        let expect_directory = index != 0 || directory;
        if (expect_directory && !metadata.is_dir()) || (!expect_directory && !metadata.is_file()) {
            bail!("Unexpected daemon resource type: {}", component.display());
        }
        check_metadata(&metadata, component)?;
    }
    Ok(resolved)
}

/// Create missing directories only beneath already protected parents.
pub fn protected_directory(path: &Path) -> Result<PathBuf> {
    match fs::symlink_metadata(path) {
        Ok(_) => protected_path(path, true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent =
                protected_directory(path.parent().context("Daemon directory has no parent")?)?;
            let target = parent.join(path.file_name().context("Daemon directory has no name")?);
            DirBuilder::new()
                .mode(0o755)
                .create(&target)
                .with_context(|| {
                    format!("Failed to create daemon directory: {}", target.display())
                })?;
            protected_path(&target, true)
        }
        Err(error) => Err(error)
            .with_context(|| format!("Failed to inspect daemon directory: {}", path.display())),
    }
}

fn write_atomic(path: &Path, content: &[u8], mode: u32) -> Result<()> {
    let mut temporary =
        tempfile::NamedTempFile::new_in(path.parent().context("Daemon file has no parent")?)
            .context("Failed to create temporary daemon file")?;
    temporary
        .write_all(content)
        .context("Failed to write daemon file")?;
    temporary
        .as_file()
        .set_permissions(fs::Permissions::from_mode(mode))
        .context("Failed to set daemon file permissions")?;
    temporary
        .as_file()
        .sync_all()
        .context("Failed to sync daemon file")?;
    // Rename replaces a pre-existing symlink/hard link instead of following it.
    temporary
        .persist(path)
        .context("Failed to replace daemon file")?;
    Ok(())
}

/// Atomically replace a config/plist without trusting an existing destination.
pub fn write_protected_file(path: &Path, content: &[u8], mode: u32) -> Result<PathBuf> {
    let parent = protected_directory(path.parent().context("Daemon file has no parent")?)?;
    let target = parent.join(path.file_name().context("Daemon file has no name")?);
    write_atomic(&target, content, mode)?;
    protected_path(&target, false)
}

/// Preserve existing logs without letting launchd follow an untrusted log path.
pub fn protected_log(path: &Path) -> Result<PathBuf> {
    let parent = protected_directory(path.parent().context("Daemon log has no parent")?)?;
    let target = parent.join(path.file_name().context("Daemon log has no name")?);
    match fs::symlink_metadata(&target) {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.nlink() != 1 {
                bail!(
                    "Daemon log must be a regular file with one link: {}",
                    target.display()
                );
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&target)
                .context("Failed to create daemon log")?;
        }
        Err(error) => return Err(error).context("Failed to inspect daemon log"),
    }
    protected_path(&target, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn rejects_non_root_and_writable_resources() {
        let path = Path::new("/resource");
        for mode in [0o600, 0o644, 0o700, 0o755] {
            assert!(check_owner_and_mode(0, mode, path).is_ok());
            assert!(check_owner_and_mode(501, mode, path).is_err());
        }
        for mode in [0o602, 0o620, 0o666, 0o775, 0o777, 0o1777] {
            assert!(check_owner_and_mode(0, mode, path).is_err());
        }
    }

    #[test]
    fn accepts_no_acl_and_standard_deny_acl() {
        let path = Path::new("/resource");
        assert!(check_acl_listing("-rw-r--r-- 1 root wheel 0 resource\n", path).is_ok());
        assert!(check_acl_listing(
            "drwxr-xr-x+ 1 root wheel 0 resource\n 0: group:everyone deny delete\n",
            path
        )
        .is_ok());
        assert!(check_acl_listing(
            "drwxr-xr-x+ 1 0 0 0 resource\n 0: ABCDEFAB-CDEF-ABCD-EFAB-CDEF0000000C inherited deny delete\n",
            path
        ).is_ok());
    }

    #[test]
    fn rejects_allow_and_unrecognized_acls() {
        let path = Path::new("/resource");
        for entry in [
            " 0: user:someone allow write,append",
            " 0: group:everyone allow add_file,delete_child",
            " 0: user:someone allow writesecurity,chown",
            " 0: user:someone inherited allow write",
            " 0: user:name deny suffix allow write",
            " unexpected ACL output",
            " 1: group:everyone deny delete",
        ] {
            assert!(check_acl_listing(&format!("drwxr-xr-x+\n{entry}\n"), path).is_err());
        }
        assert!(check_acl_listing("", path).is_err());
        assert!(check_acl_listing("drwxr-xr-x+\n", path).is_err());
    }

    #[test]
    fn retains_resolved_trusted_path_not_user_replaceable_alias() {
        let directory = tempfile::tempdir().unwrap();
        let alias = directory.path().join("alias");
        symlink("/bin/sh", &alias).unwrap();
        assert_eq!(
            protected_path(&alias, false).unwrap(),
            fs::canonicalize("/bin/sh").unwrap()
        );
    }

    #[test]
    fn rejects_paths_that_cannot_be_represented_faithfully_in_plist() {
        use std::os::unix::ffi::OsStringExt;
        for name in [vec![b'f', 0xff], b"file\rname".to_vec()] {
            let path = PathBuf::from(std::ffi::OsString::from_vec(name));
            let error = check_path_text(&path).unwrap_err().to_string();
            assert!(error.contains("invalid UTF-8") || error.contains("control characters"));
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn detects_real_macos_allow_acl_even_without_writable_mode_bits() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config");
        fs::write(&path, b"config").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let status = std::process::Command::new("/bin/chmod")
            .args(["+a", "everyone allow write"])
            .arg(&path)
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(fs::metadata(&path).unwrap().mode() & 0o022, 0);
        assert!(check_acl(&path).is_err());
    }

    #[test]
    fn rejects_writable_ancestor_even_for_read_only_file() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o777)).unwrap();
        let file = directory.path().join("listent");
        fs::write(&file, b"executable").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o555)).unwrap();
        assert!(protected_path(&file, false).is_err());
        let alias = directory.path().join("alias");
        symlink(&file, &alias).unwrap();
        assert!(protected_path(&alias, false).is_err());
    }

    #[test]
    fn does_not_create_files_under_untrusted_parent() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o777)).unwrap();
        let target = directory.path().join("new").join("config");
        assert!(write_protected_file(&target, b"configuration", 0o600).is_err());
        assert!(!directory.path().join("new").exists());
    }

    #[test]
    fn atomic_write_sets_modes_and_does_not_follow_links() {
        let directory = tempfile::tempdir().unwrap();
        let victim = directory.path().join("victim");
        fs::write(&victim, b"unchanged").unwrap();
        for (name, mode, hard_link) in [("config", 0o600, false), ("plist", 0o644, true)] {
            let target = directory.path().join(name);
            if hard_link {
                fs::hard_link(&victim, &target).unwrap();
            } else {
                symlink(&victim, &target).unwrap();
            }
            write_atomic(&target, b"new content", mode).unwrap();
            let metadata = fs::symlink_metadata(&target).unwrap();
            assert!(metadata.is_file());
            assert_eq!(metadata.mode() & 0o777, mode);
            assert_eq!(fs::read(&target).unwrap(), b"new content");
            assert_eq!(fs::read(&victim).unwrap(), b"unchanged");
            write_atomic(&target, b"replacement", mode).unwrap();
            assert_eq!(fs::read(&target).unwrap(), b"replacement");
        }
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 3);
    }

    #[test]
    fn atomic_write_creates_new_file_and_cleans_up_on_failure() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("config");
        write_atomic(&target, b"config", 0o600).unwrap();
        assert_eq!(fs::metadata(&target).unwrap().mode() & 0o777, 0o600);
        let existing_directory = directory.path().join("directory");
        fs::create_dir(&existing_directory).unwrap();
        assert!(write_atomic(&existing_directory, b"config", 0o600).is_err());
        assert!(existing_directory.is_dir());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }
}
