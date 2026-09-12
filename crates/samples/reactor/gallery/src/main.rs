#![windows_subsystem = "windows"]

mod controls;
mod pages;
mod registry;
mod router;
mod shell;

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

#[cfg(test)]
mod tests;

fn main() {
    // Development runs unpackaged and self-contained: the Windows App Runtime
    // files sit beside the executable. Toasts work for any desktop process;
    // the badge additionally needs a packaged app identity, which an MSIX
    // install provides. See packaging/ for the manifest and build script; the
    // Notifications page reports the identity error while running unpackaged.
    windows_reactor::App::run_component::<shell::Gallery>(()).unwrap();
}
