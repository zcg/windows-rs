//! Development-time package identity for the gallery.
//!
//! The badge and toast APIs on the Notifications page require a packaged app
//! identity. Starting the executable directly grants none, so `main` calls
//! [`ensure_packaged`]: an installed package with the current build is
//! activated through the shell as-is, while a missing or outdated package is
//! packed from the build output (`packaging/build-msix.ps1`) and installed
//! first. The packaged replacement launches through `shell:AppsFolder\<PFN>!App`;
//! launching the executable directly again would loop forever: activate,
//! relaunch unpackaged, activate again.
//!
//! The MSIX is framework-dependent - the build script stages no Windows App
//! Runtime files, and the manifest declares the `Microsoft.WindowsAppRuntime.2`
//! dependency. Shipping self-contained runtime files in a packaged process
//! fail-fasts in CoreMessagingXP. When packaging or installation cannot run
//! (no trusted publisher certificate, SDK tools off PATH, CI), the gallery
//! keeps running unpackaged and the Notifications page reports the API error.

use std::path::{Path, PathBuf};
use std::process::Command;

const MANIFEST: &str = "packaging/Package.appxmanifest";

/// Returns `true` when the process was started unpackaged and a packaged
/// replacement was launched (which may or may not be packaged: the caller
/// should not treat the return value as a promise of identity).
pub fn ensure_packaged() -> bool {
    if has_package_identity() {
        return false;
    }
    let manifest = workspace_root().join(MANIFEST);
    let Ok(text) = std::fs::read_to_string(&manifest) else {
        return false;
    };
    let (Ok(name), Ok(app_id)) = (
        attribute(&text, "<Identity", "Name="),
        attribute(&text, "<Application", "Id="),
    ) else {
        return false;
    };
    match activation_target(&name) {
        Ok(Some(family)) => activate(&family, &app_id),
        Ok(None) => false,
        Err(error) => {
            eprintln!("windows-reactor gallery: package activation skipped: {error}");
            false
        }
    }
}

/// Makes an identity-bearing package available and returns its family name: an
/// installed package with the current build is reused as-is, while a missing
/// or outdated one is packed from the build output and installed again. The
/// executable is newer than the packed MSIX after every code change, which is
/// the signal to repackage.
fn activation_target(name: &str) -> std::io::Result<Option<String>> {
    if let Some(family) = installed_family(name)
        && !executable_newer_than_package(name)
    {
        return Ok(Some(family));
    }
    build_and_install(name)?;
    let family = installed_family(name).ok_or_else(|| {
        std::io::Error::other("installation succeeded but the package is not registered")
    })?;
    Ok(Some(family))
}

/// Compares the built executable against the packed MSIX: `true` when the
/// executable changed after the last packaging run, or when no MSIX exists.
fn executable_newer_than_package(name: &str) -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let Ok(exe_modified) = exe.metadata().and_then(|meta| meta.modified()) else {
        return false;
    };
    let msix = workspace_root().join(format!("packaging/output/{name}.msix"));
    let Ok(msix_modified) = msix.metadata().and_then(|meta| meta.modified()) else {
        return true;
    };
    exe_modified > msix_modified
}

/// Packs the build output into the MSIX and installs it, replacing any prior
/// registration (a same-version reinstall is rejected by deployment).
fn build_and_install(name: &str) -> std::io::Result<()> {
    let script = workspace_root().join("packaging/build-msix.ps1");
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(&script)
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    let msix = workspace_root().join(format!("packaging/output/{name}.msix"));
    let script = format!(
        "Get-AppxPackage -Name '{name}' | Remove-AppxPackage -ErrorAction Stop; \
         Add-AppxPackage -Path '{}' -ErrorAction Stop",
        msix.display()
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(())
}

/// Queries the package family name registered for `name`, or `None` when the
/// identity is not registered or not installed for the current user.
fn installed_family(name: &str) -> Option<String> {
    let script = format!("(Get-AppxPackage -Name '{name}').PackageFamilyName");
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let family = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if family.is_empty() {
        None
    } else {
        Some(family)
    }
}

/// Whether this process carries a packaged app identity. Exposed so feature
/// code can pick between the system API and its unpackaged fallback.
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

/// Launches the registered application through the shell so AppModel assigns
/// it the package identity.
fn activate(family: &str, app_id: &str) -> bool {
    // The deployment queue commits the registration asynchronously; give it a
    // moment so the relaunched process observes the identity.
    std::thread::sleep(std::time::Duration::from_millis(500));
    let target = format!("shell:AppsFolder\\{family}!{app_id}");
    match Command::new("explorer.exe").arg(&target).spawn() {
        Ok(_) => true,
        Err(error) => {
            eprintln!("windows-reactor gallery: activation failed: {error}");
            false
        }
    }
}

/// Reads `key="value"` from the first occurrence of `element`.
fn attribute(text: &str, element: &str, key: &str) -> std::io::Result<String> {
    let missing = |what: &str| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("manifest has no {what}"),
        )
    };
    let start = text.find(element).ok_or_else(|| missing(element))?;
    let rest = &text[start..];
    let key_start = rest.find(key).ok_or_else(|| missing(key))?;
    let after = &rest[key_start + key.len()..];
    let quote_start = after.find('"').ok_or_else(|| missing(key))? + 1;
    let quote_end = after[quote_start..].find('"').ok_or_else(|| missing(key))?;
    Ok(after[quote_start..quote_start + quote_end].to_string())
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

    #[test]
    fn identity_and_application_attributes_are_readable() {
        let root = workspace_root();
        let manifest = root.join(MANIFEST);
        let text = std::fs::read_to_string(manifest).expect("read manifest");
        assert_eq!(
            attribute(&text, "<Identity", "Name=").unwrap(),
            "ReactorWinUIGallery"
        );
        assert_eq!(attribute(&text, "<Application", "Id=").unwrap(), "App");
    }
}
