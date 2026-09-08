#![windows_subsystem = "windows"]

mod controls;
// Generated bindings: allow style lints unconditionally so filter edits never
// trip `unfulfilled_lint_expectations` under `-D warnings`.
#[allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    clippy::upper_case_acronyms,
    clippy::too_many_arguments
)]
mod notifications_bindings;
mod packaging;
mod pages;
mod registry;
mod router;
mod shell;

#[cfg(test)]
mod tests;

fn main() {
    // The taskbar badge (BadgeNotificationManager) requires a packaged app
    // identity. In development the gallery runs unpackaged, so register the
    // build output in place (no MSIX install) when the process has no package
    // identity, then relaunch so the new process is packaged.
    if packaging::ensure_packaged() {
        return;
    }
    windows_reactor::App::run_component::<shell::Gallery>(()).unwrap();
}
