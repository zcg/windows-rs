use crate::controls::*;
use windows_reactor::*;

pub fn pips_pager_page(_: &(), cx: &mut RenderCx) -> Element {
    let (page_index, set_page_index) = cx.use_state(0i32);

    page_content(
        "PipsPager",
        "A PipsPager shows a horizontal row of pips that represent pages of content.",
        vec![
            sample_card(
                "Basic PipsPager",
                vstack((
                    pips_pager(5)
                        .selected_page_index(page_index)
                        .on_selected_index_changed(move |i| set_page_index.call(i)),
                    text_block(format!("Selected page: {}", page_index))
                        .foreground(ThemeRef::SecondaryText),
                ))
                .spacing(8.0),
                r#"pips_pager(5).selected_page_index(i).on_selected_index_changed(...)"#,
            ),
            sample_card(
                "Many Pages",
                pips_pager(30).selected_page_index(3),
                r#"pips_pager(30).selected_page_index(3)"#,
            ),
            sample_card(
                "Limited Visible Pips",
                pips_pager(20).selected_page_index(10).max_visible_pips(5),
                r#"pips_pager(20).selected_page_index(10).max_visible_pips(5)"#,
            ),
        ],
    )
}