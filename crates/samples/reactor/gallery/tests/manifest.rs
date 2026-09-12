//! Guards for `packaging/Package.appxmanifest`, which `build-msix.ps1` packs
//! and installs to give the gallery its package identity. A drift here breaks
//! the packaged identity flow.

use std::fs;

fn manifest() -> String {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("packaging/Package.appxmanifest");
    fs::read_to_string(path).expect("read packaging manifest")
}

#[test]
fn manifest_names_the_bin_target() {
    // The packaged activation launches `Executable` from the package root, so
    // it must be the file name of the built binary.
    let text = manifest();
    let bin = env!("CARGO_PKG_NAME");
    assert!(
        text.contains(&format!("Executable=\"{bin}.exe\"")),
        "manifest Executable must match {bin}.exe"
    );
}

#[test]
fn manifest_uses_an_explicit_resource_language() {
    // `x-generate` deadlocks the deployment queue for about ten minutes
    // before failing with 0x80070057.
    let text = manifest();
    assert!(
        !text.contains("x-generate"),
        "manifest must not use the invalid x-generate resource language"
    );
    assert!(text.contains("<Resource Language="));
}

#[test]
fn manifest_declares_the_expected_identity() {
    let text = manifest();
    assert!(
        text.contains("Name=\"ReactorWinUIGallery\""),
        "identity name drifted"
    );
    assert!(text.contains("Id=\"App\""), "application id drifted");
    assert!(
        text.contains("Publisher=\"CN=ReactorGalleryDev\""),
        "publisher must match the trusted development certificate"
    );
}
