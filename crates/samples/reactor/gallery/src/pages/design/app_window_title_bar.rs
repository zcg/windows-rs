use crate::controls::*;
use windows_reactor::*;

pub fn app_window_title_bar_page(_: &(), cx: &mut RenderCx) -> Element {
    let (extended, set_extended) = cx.use_state(false);
    let (accent, set_accent) = cx.use_state(false);
    let (height, set_height) = cx.use_state(String::from("Standard"));
    let (theme, set_theme) = cx.use_state(String::from("System"));

    page_content(
        "AppWindow Title Bar",
        "Customize the title bar of the current AppWindow.",
        vec![
            sample_card(
                "Extend Content into Title Bar",
                vstack((
                    ToggleSwitch::new(extended)
                        .header("Extends content")
                        .on_toggled({
                            let set_extended = set_extended.clone();
                            move |value| {
                                let mut applied = false;
                                if let Ok(title_bar) = active_title_bar() {
                                    applied = title_bar_set_extended(&title_bar, value).is_ok();
                                }
                                if applied {
                                    set_extended.call(value);
                                }
                            }
                        }),
                    text_block(if extended {
                        "App content now draws under the title bar."
                    } else {
                        "The system title bar keeps its default look."
                    })
                    .opacity(0.6),
                ))
                .spacing(8.0),
                r#"let title_bar = active_title_bar()?;
title_bar_set_extended(&title_bar, true)?"#,
            ),
            sample_card(
                "Title Bar Button Colors",
                vstack((
                    ToggleSwitch::new(accent)
                        .header("Accent buttons")
                        .on_toggled({
                            let set_accent = set_accent.clone();
                            move |value| {
                                let mut applied = false;
                                if let Ok(title_bar) = active_title_bar() {
                                    applied = title_bar_set_accent(&title_bar, value).is_ok();
                                }
                                if applied {
                                    set_accent.call(value);
                                }
                            }
                        }),
                    text_block(if accent {
                        "Minimize / maximize / close buttons use the accent color."
                    } else {
                        "Title bar buttons use the system palette."
                    })
                    .opacity(0.6),
                ))
                .spacing(8.0),
                r#"title_bar_set_accent(&title_bar, true)?  // button colors"#,
            ),
            sample_card(
                "Height and Theme",
                vstack((
                    hstack((
                        button("Standard height").on_click({
                            let set_height = set_height.clone();
                            move || {
                                if let Ok(title_bar) = active_title_bar()
                                    && title_bar_set_height(
                                        &title_bar,
                                        TitleBarHeightOption::Standard,
                                    )
                                    .is_ok()
                                {
                                    set_height.call("Standard".into());
                                }
                            }
                        }),
                        button("Tall height").on_click({
                            let set_height = set_height.clone();
                            move || {
                                if let Ok(title_bar) = active_title_bar()
                                    && title_bar_set_height(&title_bar, TitleBarHeightOption::Tall)
                                        .is_ok()
                                {
                                    set_height.call("Tall".into());
                                }
                            }
                        }),
                    ))
                    .spacing(8.0),
                    text_block(format!("Height: {height}")).opacity(0.6),
                    hstack((
                        button("System theme").on_click({
                            let set_theme = set_theme.clone();
                            move || {
                                if let Ok(title_bar) = active_title_bar()
                                    && title_bar_set_theme(
                                        &title_bar,
                                        TitleBarTheme::UseDefaultAppMode,
                                    )
                                    .is_ok()
                                {
                                    set_theme.call("System".into());
                                }
                            }
                        }),
                        button("Light theme").on_click({
                            let set_theme = set_theme.clone();
                            move || {
                                if let Ok(title_bar) = active_title_bar()
                                    && title_bar_set_theme(&title_bar, TitleBarTheme::Light).is_ok()
                                {
                                    set_theme.call("Light".into());
                                }
                            }
                        }),
                        button("Dark theme").on_click({
                            let set_theme = set_theme.clone();
                            move || {
                                if let Ok(title_bar) = active_title_bar()
                                    && title_bar_set_theme(&title_bar, TitleBarTheme::Dark).is_ok()
                                {
                                    set_theme.call("Dark".into());
                                }
                            }
                        }),
                    ))
                    .spacing(8.0),
                    text_block(format!("Theme: {theme}")).opacity(0.6),
                    button("Reset title bar").on_click(move || {
                        if let Ok(title_bar) = active_title_bar()
                            && title_bar_reset(&title_bar).is_ok()
                        {
                            set_extended.call(false);
                            set_accent.call(false);
                            set_height.call("Standard".into());
                            set_theme.call("System".into());
                        }
                    }),
                ))
                .spacing(8.0),
                r#"title_bar_set_height(&title_bar, TitleBarHeightOption::Tall)?;
title_bar_set_theme(&title_bar, TitleBarTheme::Dark)?;
title_bar_reset(&title_bar)?"#,
            ),
        ],
    )
}
