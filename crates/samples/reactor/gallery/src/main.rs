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

#[cfg(test)]
mod tests;

fn main() {
    // Development runs unpackaged and self-contained: the Windows App Runtime
    // files sit beside the executable and startup does no packaging work.
    // Toasts work for any desktop process; the badge additionally needs a
    // packaged app identity, which `packaging/build-msix.ps1 -Install`
    // provides on demand - the Notifications page shows a banner pointing
    // there while the badge reports the identity error.
    windows_reactor::App::run_component::<shell::Gallery>(()).unwrap();
}
