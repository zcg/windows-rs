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
mod pages;
mod registry;
mod router;
mod shell;

mod taskbar_badge;
#[cfg(test)]
mod tests;

fn main() {
    // The taskbar badge uses ITaskbarList3::SetOverlayIcon (taskbar_badge),
    // which needs no package identity, so the gallery runs unpackaged and
    // never registers a package.
    windows_reactor::App::run_component::<shell::Gallery>(()).unwrap();
}
