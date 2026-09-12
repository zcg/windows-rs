//! Maps a leaf destination tag to its page component. Home, Settings, the Materials design page
//! (which needs shell-owned backdrop state), and the category listing pages are resolved directly
//! by [`crate::shell`] because they need shell input or have no fixed control identity; every
//! other control destination is a self-contained component routed here.

use crate::pages::{
    basic_input, collections, date_time, design, dialogs, layout, media, menus, navigation, status,
    system, text,
};
use windows_reactor::*;

pub fn route(tag: &str) -> View {
    match tag {
        "button" => View::component::<basic_input::ButtonPage>(()),
        "check-box" => View::component::<basic_input::CheckBoxPage>(()),
        "color-picker" => View::component::<basic_input::ColorPickerPage>(()),
        "combo-box" => View::component::<basic_input::ComboBoxPage>(()),
        "drop-down-button" => View::component::<basic_input::DropDownButtonPage>(()),
        "hyperlink-button" => View::component::<basic_input::HyperlinkButtonPage>(()),
        "number-box" => View::component::<basic_input::NumberBoxPage>(()),
        "password-box" => View::component::<basic_input::PasswordBoxPage>(()),
        "radio-button" => View::component::<basic_input::RadioButtonPage>(()),
        "radio-buttons" => View::component::<basic_input::RadioButtonsPage>(()),
        "rating-control" => View::component::<basic_input::RatingControlPage>(()),
        "repeat-button" => View::component::<basic_input::RepeatButtonPage>(()),
        "slider" => View::component::<basic_input::SliderPage>(()),
        "split-button" => View::component::<basic_input::SplitButtonPage>(()),
        "text-box" => View::component::<basic_input::TextBoxPage>(()),
        "toggle-button" => View::component::<basic_input::ToggleButtonPage>(()),
        "toggle-split-button" => View::component::<basic_input::ToggleSplitButtonPage>(()),
        "toggle-switch" => View::component::<basic_input::ToggleSwitchPage>(()),

        "flip-view" => View::component::<collections::FlipViewPage>(()),
        "annotated-scroll-bar" => View::component::<collections::AnnotatedScrollBarPage>(()),
        "grid-view" => View::component::<collections::GridViewPage>(()),
        "items-repeater" => View::component::<collections::ItemsRepeaterPage>(()),
        "items-repeater-scroll-host" => {
            View::component::<collections::ItemsRepeaterScrollHostPage>(())
        }
        "item-container" => View::component::<collections::ItemContainerPage>(()),
        "items-view" => View::component::<collections::ItemsViewPage>(()),
        "list-box" => View::component::<collections::ListBoxPage>(()),
        "list-view" => View::component::<collections::ListViewPage>(()),
        "pips-pager" => View::component::<collections::PipsPagerPage>(()),
        "semantic-zoom" => View::component::<collections::SemanticZoomPage>(()),
        "swipe-control" => View::component::<collections::SwipeControlPage>(()),
        "tree-view" => View::component::<collections::TreeViewPage>(()),

        "calendar-date-picker" => View::component::<date_time::CalendarDatePickerPage>(()),
        "calendar-view" => View::component::<date_time::CalendarViewPage>(()),
        "date-picker" => View::component::<date_time::DatePickerPage>(()),
        "time-picker" => View::component::<date_time::TimePickerPage>(()),

        "typography" => View::component::<design::TypographyPage>(()),
        "color" => View::component::<design::ColorPage>(()),
        "spacing" => View::component::<design::SpacingPage>(()),
        "iconography" => View::component::<design::IconographyPage>(()),
        "geometry" => View::component::<design::GeometryPage>(()),
        "theme" => View::component::<design::ThemePage>(()),
        "system-backdrop-element" => View::component::<design::SystemBackdropElementPage>(()),

        "command-bar-flyout" => View::component::<dialogs::CommandBarFlyoutPage>(()),
        "content-dialog" => View::component::<dialogs::ContentDialogPage>(()),
        "flyout" => View::component::<dialogs::FlyoutPage>(()),
        "menu-flyout" => View::component::<dialogs::MenuFlyoutPage>(()),
        "teaching-tip" => View::component::<dialogs::TeachingTipPage>(()),

        "border" => View::component::<layout::BorderPage>(()),
        "canvas" => View::component::<layout::CanvasPage>(()),
        "expander" => View::component::<layout::ExpanderPage>(()),
        "grid" => View::component::<layout::GridPage>(()),
        "parallax-view" => View::component::<layout::ParallaxViewPage>(()),
        "relative-panel" => View::component::<layout::RelativePanelPage>(()),
        "scroll-view" => View::component::<layout::ScrollViewPage>(()),
        "scroll-viewer" => View::component::<layout::ScrollViewerPage>(()),
        "split-view" => View::component::<layout::SplitViewPage>(()),
        "stack-panel" => View::component::<layout::StackPanelPage>(()),
        "swap-chain-panel" => View::component::<layout::SwapChainPanelPage>(()),
        "two-pane-view" => View::component::<layout::TwoPaneViewPage>(()),
        "variable-sized-wrap-grid" => View::component::<layout::VariableSizedWrapGridPage>(()),
        "viewbox" => View::component::<layout::ViewboxPage>(()),

        "image" => View::component::<media::ImagePage>(()),
        "icon-element" => View::component::<media::IconElementPage>(()),
        "map-control" => View::component::<media::MapControlPage>(()),
        "media-player-element" => View::component::<media::MediaPlayerElementPage>(()),
        "media-transport-controls" => View::component::<media::MediaTransportControlsPage>(()),
        "animated-visual-player" => View::component::<media::AnimatedVisualPlayerPage>(()),
        "animated-icon" => View::component::<media::AnimatedIconPage>(()),
        "person-picture" => View::component::<media::PersonPicturePage>(()),
        "shape" => View::component::<media::ShapePage>(()),
        "webview" => View::component::<media::WebViewPage>(()),

        "command-bar" => View::component::<menus::CommandBarPage>(()),
        "app-bar-button" => View::component::<menus::AppBarButtonPage>(()),
        "app-bar-separator" => View::component::<menus::AppBarSeparatorPage>(()),
        "app-bar-toggle-button" => View::component::<menus::AppBarToggleButtonPage>(()),
        "app-bar-element-container" => View::component::<menus::AppBarElementContainerPage>(()),
        "menu-bar" => View::component::<menus::MenuBarPage>(()),
        "selector-bar" => View::component::<menus::SelectorBarPage>(()),

        "breadcrumb-bar" => View::component::<navigation::BreadcrumbBarPage>(()),
        "frame" => View::component::<navigation::FramePage>(()),
        "navigation-view" => View::component::<navigation::NavigationViewPage>(()),
        "pivot" => View::component::<navigation::PivotPage>(()),
        "tab-view" => View::component::<navigation::TabViewPage>(()),
        "title-bar" => View::component::<navigation::TitleBarPage>(()),

        "info-badge" => View::component::<status::InfoBadgePage>(()),
        "info-bar" => View::component::<status::InfoBarPage>(()),
        "progress-bar" => View::component::<status::ProgressBarPage>(()),
        "progress-ring" => View::component::<status::ProgressRingPage>(()),
        "refresh-container" => View::component::<status::RefreshContainerPage>(()),
        "refresh-visualizer" => View::component::<status::RefreshVisualizerPage>(()),
        "tool-tip" => View::component::<status::ToolTipPage>(()),

        "auto-suggest-box" => View::component::<text::AutoSuggestBoxPage>(()),
        "rich-edit-box" => View::component::<text::RichEditBoxPage>(()),
        "rich-text-block" => View::component::<text::RichTextBlockPage>(()),
        "text-block" => View::component::<text::TextBlockPage>(()),
        "type-ramp" => View::component::<text::TypeRampPage>(()),

        "camera" => View::component::<system::CameraPage>(()),
        "clipboard" => View::component::<system::ClipboardPage>(()),
        "jump-list" => View::component::<system::JumpListPage>(()),
        "multiple-windows" => View::component::<system::MultipleWindowsPage>(()),
        "notifications" => View::component::<system::NotificationsPage>(()),
        "storage-pickers" => View::component::<system::StoragePickersPage>(()),
        "transitions" => View::component::<system::TransitionsPage>(()),

        _ => TextBlock::new()
            .text(format!("Page not found: {tag}"))
            .opacity(0.6)
            .into(),
    }
}
