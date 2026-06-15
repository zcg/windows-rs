use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const RUNTIME_PKG: &str = "Microsoft.WindowsAppSDK.Runtime";
const RUNTIME_VER: &str = "2.1.3";
const RUNTIME_FILES: &str = include_str!("../assets/runtime.txt");
const APP_MANIFEST: &str = include_str!("../assets/app.manifest");
const NUGET_URL: &str = "https://www.nuget.org/api/v2/package/{name}/{version}";
const RESOURCES_PRI: &str = "resources.pri";
const XAML_PRI: &str = "Microsoft.UI.pri";
const BOOTSTRAP_DLL: &str = "Microsoft.WindowsAppRuntime.Bootstrap.dll";
const STAGING_LOCK_TIMEOUT: Duration = Duration::from_secs(600);
const RUNTIME_SHA512: &str = "f93b6325b334e3a819e9972f177054b170de400c5b0284bd3ec440ebf6220712932021819f81172447ea0b7967c8f8b5add63a7c9d56cf9392927b3df6a61d1e";
const RUNTIME_HASH_SOURCE: &str = "https://api.nuget.org/v3/catalog0/data/2026.05.21.22.56.05/microsoft.windowsappsdk.runtime.2.1.3.json";

fn assert_windows() {
    match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("windows") => {}
        Ok(os) => panic!("unsupported target OS: {os}"),
        Err(_) => panic!("CARGO_CFG_TARGET_OS not set"),
    }
}

/// Configures the app to run with a Windows App Runtime dependency.
pub fn as_framework_dependent() {
    assert_windows();
    as_framework_dependent_impl("");
}

/// Configures an example to run with a Windows App Runtime dependency.
pub fn as_example() {
    assert_windows();
    as_framework_dependent_impl("examples");
}

fn as_framework_dependent_impl(subdir: &str) {
    let out_dir = out_dir();
    let dest = if subdir.is_empty() {
        target_dir_from_out(&out_dir)
    } else {
        target_dir_from_out(&out_dir).join(subdir)
    };
    copy_bootstrap_to(&dest);
    let temp_dir = temp_dir();
    let runtime = stage_runtime(&temp_dir);
    let extract = ensure_msix_extracted(&runtime, &[XAML_PRI]);
    track_output_file(&dest.join(BOOTSTRAP_DLL));
    track_output_file(&dest.join(RESOURCES_PRI));
    copy_file(&extract.join(XAML_PRI), &dest, RESOURCES_PRI);
}

/// Configures the app to run completely self-contained.
pub fn as_self_contained() {
    assert_windows();

    let out_dir = out_dir();
    let temp_dir = temp_dir();
    let runtime = stage_runtime(&temp_dir);
    let required_entries = runtime_entries().collect::<Vec<_>>();
    let extract = ensure_msix_extracted(&runtime, &required_entries);
    let dest = target_dir_from_out(&out_dir);
    track_runtime_outputs(&dest);
    copy_runtime_to(&extract, &dest);

    let manifest_path = out_dir.join("app.manifest");
    fs::write(&manifest_path, APP_MANIFEST).unwrap_or_else(|e| {
        panic!(
            "failed to write manifest to {}: {e}",
            manifest_path.display()
        )
    });
    let target_env = env::var("CARGO_CFG_TARGET_ENV").expect("CARGO_CFG_TARGET_ENV not set");
    let target_abi = env::var("CARGO_CFG_TARGET_ABI").unwrap_or_default();
    match (target_env.as_str(), target_abi.as_str()) {
        ("msvc", _) => {
            println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
            println!(
                "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
                manifest_path.display()
            );
        }
        ("gnu", "llvm") => {
            println!("cargo:rustc-link-arg-bins=-Wl,/MANIFEST:EMBED");
            println!(
                "cargo:rustc-link-arg-bins=-Wl,/MANIFESTINPUT:{}",
                manifest_path.display()
            );
        }
        _ => panic!("unsupported target environment: {target_env}{target_abi}"),
    }
}

fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"))
}

fn temp_dir() -> PathBuf {
    let base = if let Some(p) = env::var_os("LOCALAPPDATA") {
        PathBuf::from(p)
    } else if let Some(p) = env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(p)
    } else if let Some(p) = env::var_os("HOME") {
        PathBuf::from(p).join(".cache")
    } else {
        panic!(
            "could not determine cache directory: LOCALAPPDATA, XDG_CACHE_HOME, and HOME are all unset"
        );
    };
    let temp = base.join("windows-reactor-setup").join("temp");
    fs::create_dir_all(&temp)
        .unwrap_or_else(|e| panic!("failed to create temp directory {}: {e}", temp.display()));
    temp
}

fn copy_bootstrap_to(dest: &Path) {
    let bytes: &[u8] = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86") => include_bytes!("../bootstrap/x86/Microsoft.WindowsAppRuntime.Bootstrap.dll"),
        Ok("x86_64") => {
            include_bytes!("../bootstrap/x64/Microsoft.WindowsAppRuntime.Bootstrap.dll")
        }
        Ok("aarch64") => {
            include_bytes!("../bootstrap/arm64/Microsoft.WindowsAppRuntime.Bootstrap.dll")
        }
        Ok("arm64ec") => {
            include_bytes!("../bootstrap/x64/Microsoft.WindowsAppRuntime.Bootstrap.dll")
        }
        Ok(arch) => panic!("Unsupported bootstrap target architecture: {arch}"),
        Err(_) => panic!("CARGO_CFG_TARGET_ARCH not set"),
    };
    fs::create_dir_all(dest)
        .unwrap_or_else(|e| panic!("failed to create directory {}: {e}", dest.display()));
    let dest = dest.join(BOOTSTRAP_DLL);
    fs::write(&dest, bytes).unwrap_or_else(|e| panic!("failed to write {}: {e}", dest.display()));
}

fn ensure_msix_extracted(runtime: &Path, required_entries: &[&str]) -> PathBuf {
    let arch = format!("win10-{}", target_arch());
    let msix = runtime
        .join("MSIX")
        .join(&arch)
        .join("Microsoft.WindowsAppRuntime.2.msix");
    let extract = runtime.join(format!(".msix_extract-{arch}"));
    let _lock = stage_lock(&runtime.join(format!(".msix_extract-{arch}.lock")));

    if !required_entries_exist(&extract, required_entries) {
        recreate_dir(&extract);
        extract_tar(&msix, &extract, &[])
            .unwrap_or_else(|e| panic!("failed to extract {}: {e}", msix.display()));
    }
    for entry in required_entries {
        assert_runtime_entry_exists(&extract.join(entry), entry);
    }

    extract
}

fn copy_runtime_to(src: &Path, dest: &Path) {
    for name in runtime_entries() {
        let path = src.join(name);
        if path.is_file() {
            copy_file(&path, dest, name);
        } else if path.is_dir() {
            let sub = dest.join(name);
            fs::create_dir_all(&sub)
                .unwrap_or_else(|e| panic!("failed to create directory {}: {e}", sub.display()));
            copy_dir_contents(&path, &sub);
        } else {
            panic!("runtime dependency {name} not found at {}", path.display());
        }
    }
}

fn copy_dir_contents(src: &Path, dest: &Path) {
    let entries = fs::read_dir(src)
        .unwrap_or_else(|e| panic!("failed to read directory {}: {e}", src.display()));
    for entry in entries {
        let entry = entry
            .unwrap_or_else(|e| panic!("failed to read directory entry in {}: {e}", src.display()));
        let path = entry.path();
        let Some(name) = entry.file_name().into_string().ok() else {
            continue;
        };
        if path.is_file() {
            copy_file(&path, dest, &name);
        } else if path.is_dir() {
            let sub = dest.join(&name);
            fs::create_dir_all(&sub)
                .unwrap_or_else(|e| panic!("failed to create directory {}: {e}", sub.display()));
            copy_dir_contents(&path, &sub);
        }
    }
}

fn copy_file(src: &Path, base: &Path, name: &str) {
    assert_file_exists(src, name);
    fs::create_dir_all(base)
        .unwrap_or_else(|e| panic!("failed to create directory {}: {e}", base.display()));
    let dest = base.join(name);
    if files_match(src, &dest) {
        return;
    }
    fs::copy(src, &dest).unwrap_or_else(|e| {
        panic!(
            "failed to copy {} to {}: {e}",
            src.display(),
            dest.display()
        )
    });
}

fn files_match(first: &Path, second: &Path) -> bool {
    let Ok(first_metadata) = fs::metadata(first) else {
        return false;
    };
    let Ok(second_metadata) = fs::metadata(second) else {
        return false;
    };
    if !first_metadata.is_file()
        || !second_metadata.is_file()
        || first_metadata.len() != second_metadata.len()
    {
        return false;
    }
    match (fs::read(first), fs::read(second)) {
        (Ok(first), Ok(second)) => first == second,
        _ => false,
    }
}

fn track_output_file(path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());
}

fn track_runtime_outputs(dest: &Path) {
    for name in runtime_entries() {
        track_output_file(&dest.join(name));
    }
}

fn runtime_entries() -> impl Iterator<Item = &'static str> {
    RUNTIME_FILES
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

fn stage_runtime(temp: &Path) -> PathBuf {
    let arch = format!("win10-{}", target_arch());
    stage_pkg(
        RUNTIME_PKG,
        RUNTIME_VER,
        temp,
        &["MSIX", &arch, "Microsoft.WindowsAppRuntime.2.msix"],
        RUNTIME_SHA512,
        RUNTIME_HASH_SOURCE,
    )
}

fn stage_pkg(
    name: &str,
    ver: &str,
    temp: &Path,
    required_file: &[&str],
    expected_sha512: &str,
    hash_source: &str,
) -> PathBuf {
    let nupkg = temp.join(format!("{name}.{ver}.nupkg"));
    let extract = temp.join(format!("{name}-{ver}"));
    let _lock = stage_lock(&temp.join(format!("{name}-{ver}.lock")));

    for attempt in 0..2 {
        if !is_file_with_data(&nupkg) {
            dl_nupkg(name, ver, &nupkg);
        }
        let actual_sha512 = file_sha512(&nupkg)
            .unwrap_or_else(|e| panic!("failed to verify {}: {e}", nupkg.display()));
        if !actual_sha512.eq_ignore_ascii_case(expected_sha512) {
            if attempt == 0 {
                fs::remove_file(&nupkg).unwrap_or_else(|e| {
                    panic!("failed to remove corrupt nupkg {}: {e}", nupkg.display())
                });
                continue;
            }
            panic!(
                "SHA512 mismatch for {}: expected {expected_sha512} from {hash_source}, got {actual_sha512}",
                nupkg.display()
            );
        }
        let required = required_path(&extract, required_file);
        if !is_file_with_data(&required) {
            recreate_dir(&extract);
            match extract_tar(&nupkg, &extract, &["--strip-components=1"]) {
                Ok(()) => {}
                Err(_) if attempt == 0 => {
                    fs::remove_file(&nupkg).unwrap_or_else(|e| {
                        panic!("failed to remove corrupt nupkg {}: {e}", nupkg.display())
                    });
                    continue;
                }
                Err(err) => panic!("failed to extract {name} {ver}: {err}"),
            }
        }
        let required = required_path(&extract, required_file);
        if is_file_with_data(&required) {
            return extract;
        }
        fs::remove_file(&nupkg)
            .unwrap_or_else(|e| panic!("failed to remove nupkg {}: {e}", nupkg.display()));
    }
    panic!(
        "failed to stage {name} {ver}: required file {} was not found",
        required_path(&extract, required_file).display()
    )
}

struct StageLock {
    path: PathBuf,
    file: Option<File>,
}

impl Drop for StageLock {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}

fn stage_lock(path: &Path) -> StageLock {
    let start = Instant::now();
    loop {
        match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(mut file) => {
                write_lock_owner(&mut file);
                return StageLock {
                    path: path.to_path_buf(),
                    file: Some(file),
                };
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                if remove_stale_lock(path) {
                    continue;
                }
                assert!(
                    start.elapsed() <= STAGING_LOCK_TIMEOUT,
                    "timed out waiting for staging lock {}",
                    path.display()
                );
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => panic!("failed to create staging lock {}: {e}", path.display()),
        }
    }
}

fn write_lock_owner(file: &mut File) {
    let created_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    writeln!(file, "pid={}", process::id()).expect("failed to write staging lock pid");
    writeln!(file, "created_unix_ms={created_unix_ms}")
        .expect("failed to write staging lock timestamp");
}

fn remove_stale_lock(path: &Path) -> bool {
    let owner = lock_owner(path);
    let Some(age) = owner
        .as_ref()
        .and_then(|owner| lock_age_from_created(owner.created_unix_ms))
        .or_else(|| lock_age_from_metadata(path))
    else {
        return false;
    };

    if age <= STAGING_LOCK_TIMEOUT {
        return false;
    }
    if owner
        .as_ref()
        .and_then(|owner| process_exists(owner.pid))
        .unwrap_or(false)
    {
        return false;
    }
    match fs::remove_file(path) {
        Ok(()) => true,
        Err(e) if e.kind() == ErrorKind::NotFound => true,
        Err(e) => panic!(
            "failed to remove stale staging lock {}: {e}",
            path.display()
        ),
    }
}

struct LockOwner {
    pid: u32,
    created_unix_ms: Option<u128>,
}

fn lock_owner(path: &Path) -> Option<LockOwner> {
    let contents = fs::read_to_string(path).ok()?;
    let pid = contents
        .lines()
        .find_map(|line| line.strip_prefix("pid=")?.parse().ok())?;
    let created_unix_ms = contents
        .lines()
        .find_map(|line| line.strip_prefix("created_unix_ms=")?.parse().ok());
    Some(LockOwner {
        pid,
        created_unix_ms,
    })
}

fn lock_age_from_created(created_unix_ms: Option<u128>) -> Option<Duration> {
    let created_unix_ms = created_unix_ms?;
    let now_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis();
    let age = now_unix_ms.saturating_sub(created_unix_ms);
    Some(Duration::from_millis(age.min(u128::from(u64::MAX)) as u64))
}

fn lock_age_from_metadata(path: &Path) -> Option<Duration> {
    fs::metadata(path).ok()?.modified().ok()?.elapsed().ok()
}

fn process_exists(pid: u32) -> Option<bool> {
    let tasklist = system_tool("tasklist.exe")?;
    let filter = format!("PID eq {pid}");
    let Ok(output) = Command::new(tasklist)
        .args(["/FI", &filter, "/FO", "CSV", "/NH"])
        .output()
    else {
        return None;
    };
    if !output.status.success() {
        return None;
    }
    let pid_field = format!(",\"{pid}\",");
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.contains(&pid_field)),
    )
}

fn dl_nupkg(name: &str, ver: &str, dest: &Path) {
    let url = NUGET_URL.replace("{name}", name).replace("{version}", ver);
    let temp = dest.with_extension("nupkg.download");
    println!("Downloading {name} {ver}");
    let curl = system_tool("curl.exe")
        .unwrap_or_else(|| panic!("curl.exe not found under SystemRoot; cannot download {url}"));
    let output = Command::new(&curl)
        .args(["-sS", "-f", "-L", "--retry", "3", "-o"])
        .arg(&temp)
        .arg(&url)
        .output()
        .unwrap_or_else(|e| panic!("failed to start {}: {e}", curl.display()));
    if !output.status.success() {
        if temp.is_file() {
            fs::remove_file(&temp)
                .unwrap_or_else(|e| panic!("failed to remove temp file {}: {e}", temp.display()));
        }
        panic!(
            "failed to download {name} {ver} from {url}: {}",
            command_output(&output)
        );
    }
    assert_file_exists(&temp, "downloaded package");
    if dest.exists() {
        fs::remove_file(dest)
            .unwrap_or_else(|e| panic!("failed to replace {}: {e}", dest.display()));
    }
    fs::rename(&temp, dest).unwrap_or_else(|e| {
        panic!(
            "failed to move {} to {}: {e}",
            temp.display(),
            dest.display()
        )
    });
    println!("Downloaded {name} {ver}");
}

fn extract_tar(src: &Path, dst: &Path, extra: &[&str]) -> Result<(), String> {
    assert_file_exists(src, "archive");
    println!("Extracting {} to {}", src.display(), dst.display());
    let tar =
        system_tool("tar.exe").ok_or_else(|| "tar.exe not found under SystemRoot".to_string())?;
    let output = Command::new(&tar)
        .arg("-xf")
        .arg(src)
        .arg("-C")
        .arg(dst)
        .args(extra)
        .output()
        .map_err(|e| format!("failed to start {}: {e}", tar.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_output(&output))
    }
}

fn file_sha512(path: &Path) -> Result<String, String> {
    let certutil = system_tool("certutil.exe")
        .ok_or_else(|| "certutil.exe not found under SystemRoot".to_string())?;
    let output = Command::new(&certutil)
        .args(["-hashfile"])
        .arg(path)
        .arg("SHA512")
        .output()
        .map_err(|e| format!("failed to start {}: {e}", certutil.display()))?;
    if !output.status.success() {
        return Err(command_output(&output));
    }
    parse_certutil_hash(&output.stdout).ok_or_else(|| {
        format!(
            "failed to parse SHA512 from certutil output for {}: {}",
            path.display(),
            command_output(&output)
        )
    })
}

fn parse_certutil_hash(stdout: &[u8]) -> Option<String> {
    String::from_utf8_lossy(stdout).lines().find_map(|line| {
        let hash = line
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        if hash.len() == 128 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            Some(hash.to_ascii_lowercase())
        } else {
            None
        }
    })
}

fn system_tool(name: &str) -> Option<PathBuf> {
    env::var_os("SystemRoot")
        .map(|r| PathBuf::from(r).join("System32").join(name))
        .filter(|p| p.is_file())
}

fn is_file_with_data(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0)
}

fn required_path(base: &Path, parts: &[&str]) -> PathBuf {
    let mut path = base.to_path_buf();
    for part in parts {
        path = path.join(part);
    }
    path
}

fn required_entries_exist(dir: &Path, entries: &[&str]) -> bool {
    entries
        .iter()
        .all(|entry| runtime_entry_exists(&dir.join(entry)))
}

fn recreate_dir(path: &Path) {
    if path.is_dir() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|e| panic!("failed to remove directory {}: {e}", path.display()));
    } else if path.exists() {
        fs::remove_file(path)
            .unwrap_or_else(|e| panic!("failed to remove file {}: {e}", path.display()));
    }
    fs::create_dir_all(path)
        .unwrap_or_else(|e| panic!("failed to create directory {}: {e}", path.display()));
}

fn assert_file_exists(path: &Path, name: &str) {
    assert!(
        is_file_with_data(path),
        "{name} not found at {}",
        path.display()
    );
}

fn assert_runtime_entry_exists(path: &Path, name: &str) {
    assert!(
        runtime_entry_exists(path),
        "Windows App Runtime entry {name} not found at {}",
        path.display()
    );
}

fn runtime_entry_exists(path: &Path) -> bool {
    path.is_dir() || is_file_with_data(path)
}

fn command_output(output: &Output) -> String {
    let mut msg = String::new();
    if let Some(code) = output.status.code() {
        msg.push_str(&format!("exit code {code}"));
    } else {
        msg.push_str("terminated by signal");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        msg.push_str(&format!(": stdout: {stdout}"));
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        msg.push_str(&format!(": stderr: {stderr}"));
    }
    msg
}

fn target_dir_from_out(out: &Path) -> PathBuf {
    out.ancestors().nth(3).unwrap_or(out).to_path_buf()
}

fn target_arch() -> &'static str {
    match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("aarch64") => "arm64",
        Ok("arm64ec") => "arm64ec",
        Ok("x86") => "x86",
        Ok("x86_64") => "x64",
        Ok(arch) => panic!("Unsupported Windows App Runtime target architecture: {arch}"),
        Err(_) => panic!("CARGO_CFG_TARGET_ARCH not set"),
    }
}
