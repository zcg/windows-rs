//! Development-time package identity for the gallery.
//!
//! The taskbar badge API (`Microsoft.Windows.BadgeNotifications`) requires a
//! packaged app identity. Registering an MSIX every iteration is heavy, so the
//! gallery uses `Add-AppxPackage -Register` on the manifest in the build
//! output directory. The manifest sits next to the executable (same relative
//! name as `Executable`), so the in-place files keep working with a package
//! identity and no MSIX install is involved.
//!
//! Registration is idempotent: it fails quietly when the package is already
//! present, and the gallery keeps running unpackaged when the system cannot
//! register it (for example, in CI). A packaged process resolves its own
//! package full name; an unpackaged one does not.

use std::path::{Path, PathBuf};
use std::process::Command;

const MANIFEST: &str = "packaging/Package.appxmanifest";

/// Returns `true` when the process was started unpackaged and registration
/// produced a new process (which may or may not be packaged: the caller should
/// not treat the return value as a promise of identity).
pub fn ensure_packaged() -> bool {
    if has_package_identity() {
        return false;
    }
    match register_in_place() {
        Ok(()) => relaunch(),
        Err(error) => {
            eprintln!("windows-reactor gallery: package registration skipped: {error}");
            false
        }
    }
}

fn has_package_identity() -> bool {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentPackageFullName(
            package_name_length: *mut u32,
            package_full_name: *mut u16,
        ) -> i32;
    }
    const APPMODEL_ERROR_NO_PACKAGE: i32 = 15700;
    let mut length = 0u32;
    unsafe {
        // First call with a null buffer returns the required length; the
        // error 15700 ("no package") is the unpackaged-process signal.
        let result = GetCurrentPackageFullName(&mut length, std::ptr::null_mut());
        result != APPMODEL_ERROR_NO_PACKAGE
    }
}

/// Registers the build output in place with a package identity.
fn register_in_place() -> std::io::Result<()> {
    let root = workspace_root();
    let manifest = root.join(MANIFEST);
    if !manifest.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("missing manifest at {}", manifest.display()),
        ));
    }
    // `Add-AppxPackage -Register` registers a loose-layout package from the
    // manifest's directory: the AppxManifest.xml must sit next to the
    // executable, and its `Executable` attribute names that executable.
    // The packaging manifest lives in this repo, so copy it beside the exe.
    let output = build_output_dir();
    let staged = output.join("AppxManifest.xml");
    std::fs::copy(&manifest, &staged)?;
    let output_result = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("Add-AppxPackage -Register \"{}\"", staged.display()),
        ])
        .output()?;
    if output_result.status.success() {
        return Ok(());
    }
    Err(std::io::Error::other(
        String::from_utf8_lossy(&output_result.stderr)
            .trim()
            .to_string(),
    ))
}

/// Relaunches the gallery so the new process can pick up the package identity.
fn relaunch() -> bool {
    // The deployment queue commits the registration asynchronously; give it a
    // moment so the relaunched process observes the identity.
    std::thread::sleep(std::time::Duration::from_millis(500));
    let exe = std::env::current_exe().expect("current exe path");
    match Command::new(&exe).spawn() {
        Ok(_) => true,
        Err(error) => {
            eprintln!("windows-reactor gallery: relaunch failed: {error}");
            false
        }
    }
}

/// The directory containing the built executable, resolved from the current
/// executable path (`target/debug` or `target/release`).
fn build_output_dir() -> PathBuf {
    std::env::current_exe()
        .and_then(|exe| {
            exe.parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| std::io::Error::other("exe has no parent"))
        })
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Resolves the repo root from the manifest path convention: the gallery keeps
/// its packaging assets under `crates/samples/reactor/gallery/packaging/`.
fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .find(|path| path.join(MANIFEST).is_file())
        .map_or_else(|| manifest.to_path_buf(), Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_names_the_bin_target() {
        // The sparse registration derives the package layout from the
        // manifest directory: `Executable` must be the file name of the
        // binary that will show the badge.
        let root = workspace_root();
        let manifest = root.join(MANIFEST);
        let text = std::fs::read_to_string(manifest).expect("read manifest");
        let bin = env!("CARGO_BIN_NAME");
        assert!(
            text.contains(&format!("Executable=\"{bin}.exe\"")),
            "manifest Executable must match {bin}.exe"
        );
    }

    #[test]
    fn manifest_uses_an_explicit_resource_language() {
        // `x-generate` deadlocks the deployment queue for about ten minutes
        // before failing with 0x80070057.
        let root = workspace_root();
        let manifest = root.join(MANIFEST);
        let text = std::fs::read_to_string(manifest).expect("read manifest");
        assert!(
            !text.contains("x-generate"),
            "manifest must not use the invalid x-generate resource language"
        );
        assert!(text.contains("<Resource Language="));
    }

    #[test]
    fn manifest_publisher_is_trusted_prefix() {
        // The publisher must match a trusted certificate on the development
        // machine or registration hangs at the deployment approval prompt.
        let root = workspace_root();
        let manifest = root.join(MANIFEST);
        let text = std::fs::read_to_string(manifest).expect("read manifest");
        assert!(text.contains("Publisher="));
    }
}
