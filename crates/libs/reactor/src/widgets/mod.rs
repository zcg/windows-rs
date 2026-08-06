//! Built-in widget catalog. Each file defines a widget, its builders,
//! `Widget` implementation, support types, and constructor.

// Make the core types each widget module depends on visible via
// `use super::*;` inside the per-widget files.
pub use super::*;

/// Emit the boilerplate `kind`/`key`/`modifiers` accessors shared by
/// every `impl Widget for X`; `attached` falls through to the default
/// impl on the trait.
macro_rules! widget_header {
    ($kind:expr) => {
        fn kind(&self) -> ControlKind {
            $kind
        }
        fn key(&self) -> Option<&str> {
            self.key.as_deref()
        }
        fn modifiers(&self) -> &Modifiers {
            &self.modifiers
        }
    };
}
pub(crate) use widget_header;

/// Declare a widget submodule and re-export its public items.
macro_rules! widget_modules {
    ($($mod:ident),* $(,)?) => {
        $( pub mod $mod; )*
        $( pub use $mod::*; )*
    };
}

widget_modules! {
    annotated_scroll_bar,
    auto_suggest_box,
    border,
    breadcrumb_bar,
    button,
    calendar_date_picker,
    calendar_view,
    canvas,
    check_box,
    color_picker,
    combo_box,
    command_bar,
    composition_host,
    content_dialog,
    date_picker,
    drop_down_button,
    expander,
    flyout,
    grid,
    hyperlink_button,
    icon,
    image,
    info_badge,
    info_bar,
    list_box,
    media_player_element,
    menu_bar,
    navigation_view,
    number_box,
    parallax_view,
    password_box,
    person_picture,
    pips_pager,
    pivot,
    progress_bar,
    progress_ring,
    radio_button,
    radio_buttons,
    rating_control,
    refresh_container,
    relative_panel,
    repeat_button,
    rich_edit_box,
    scroll_view,
    scroll_viewer,
    selector_bar,
    shape,
    slider,
    split_button,
    split_view,
    swipe_control,
    stack_panel,
    surface_image_source,
    swap_chain_panel,
    tab_view,
    teaching_tip,
    text_block,
    text_box,
    time_picker,
    title_bar,
    toggle_split_button,
    toggle_button,
    toggle_switch,
    tree_view,
    variable_sized_wrap_grid,
    viewbox,
    web_view2,
}
