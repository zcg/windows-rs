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
    // The badge and toast APIs on the Notifications page are the official
    // Windows App SDK ones and require a packaged app identity. A direct
    // executable launch therefore packs the current build into the MSIX
    // (installed on first run, refreshed when the build changed) and
    // relaunches through the package; an already-current install is
    // activated without reinstalling.
    if packaging::ensure_packaged() {
        return;
    }
    windows_reactor::App::run_component::<shell::Gallery>(()).unwrap();
}
