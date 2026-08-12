use windows_reactor::*;

pub fn settings_page(_: &(), cx: &mut RenderCx) -> Element {
    let (theme_label, set_theme_label) = cx.use_state(String::from("System"));
    let scheme = cx.use_color_scheme();

    let content: Element = vstack((
        text_block("Settings").font_size(28.0).bold(),
        vstack((
            text_block("Theme").bold(),
            vstack((
                hstack((
                    button("System").on_click({
                        let set_theme_label = set_theme_label.clone();
                        move || {
                            set_requested_theme(RequestedTheme::Default);
                            set_theme_label.call("System".into());
                        }
                    }),
                    button("Light").on_click({
                        let set_theme_label = set_theme_label.clone();
                        move || {
                            set_requested_theme(RequestedTheme::Light);
                            set_theme_label.call("Light".into());
                        }
                    }),
                    button("Dark").on_click(move || {
                        set_requested_theme(RequestedTheme::Dark);
                        set_theme_label.call("Dark".into());
                    }),
                ))
                .spacing(8.0),
                text_block(format!(
                    "Requested: {theme_label} | Effective: {}",
                    if matches!(scheme, ColorScheme::Dark) { "Dark" } else { "Light" }
                ))
                .font_size(13.0)
                .opacity(0.6),
            ))
            .spacing(8.0),
        ))
        .spacing(12.0)
        .padding(20.0),
        vstack((
            text_block("About this app").bold(),
            vstack((
                text_block("WinUI Gallery (Reactor)").bold(),
                text_block("Version 0.1.0").font_size(12.0).opacity(0.6),
            ))
            .spacing(2.0),
            text_block(
                "This app is built with Reactor, a declarative component-based UI framework for WinUI 3. It demonstrates how to recreate the WinUI Gallery experience using reactive hooks and a composable element DSL."
            )
            .font_size(13.0)
            .wrap()
            .opacity(0.6)
        ))
        .spacing(12.0)
        .padding(20.0),
        vstack((
            text_block("Built with Reactor").bold(),
            text_block("Framework: Reactor (declarative Rust DSL)").font_size(13.0),
            text_block("Platform: WinUI 3 / Windows App SDK").font_size(13.0),
            text_block("Rendering: Virtual DOM reconciler").font_size(13.0),
            text_block("State: Hook-based state management").font_size(13.0)
        ))
        .spacing(8.0)
        .padding(20.0)
    ))
    .spacing(24.0)
    .margin(Thickness {
        left: 36.0,
        top: 24.0,
        right: 36.0,
        bottom: 36.0,
    })
    .horizontal_alignment(HorizontalAlignment::Stretch)
    .into();
    scroll_view(content).into()
}
