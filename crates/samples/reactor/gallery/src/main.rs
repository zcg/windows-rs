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
    // The default run is self-contained: the Windows App Runtime files sit
    // beside the executable and nothing outside the build output is touched.
    //
    // The badge and toast APIs on the Notifications page additionally need a
    // packaged app identity, which only a package activation provides. That
    // flow is opt-in through `--packaged`: it registers the build output in
    // place (or activates the installed MSIX) and relaunches packaged. On a
    // Windows Insider build (26340) every packaged launch currently
    // fail-fasts in CoreMessagingXP regardless of packaging style, so the
    // flag stays opt-in until that is resolved.
    if std::env::args().any(|argument| argument == "--packaged") && packaging::ensure_packaged() {
        return;
    }
    windows_reactor::App::run_component::<shell::Gallery>(()).unwrap();
}
