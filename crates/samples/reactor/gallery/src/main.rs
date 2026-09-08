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
    // The system badge API (Windows.UI.Notifications.BadgeUpdateManager)
    // requires package identity. On first launch the gallery registers its
    // build output in place and relaunches through the package, so the new
    // process is packaged without installing an MSIX.
    if packaging::ensure_packaged() {
        return;
    }
    windows_reactor::App::run_component::<shell::Gallery>(()).unwrap();
}
