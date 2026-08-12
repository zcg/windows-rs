use crate::controls::*;
use windows_reactor::*;

fn tile(label: &str, r: u8, g: u8, b: u8, row_span: i32, column_span: i32) -> Element {
    border(
        text_block(label)
            .font_size(13.0)
            .foreground(Color {
                a: 255,
                r: 255,
                g: 255,
                b: 255,
            })
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center),
    )
    .background(Color { a: 255, r, g, b })
    .corner_radius(4.0)
    .margin(Thickness::uniform(2.0))
    .wrap_grid_row_span(row_span)
    .wrap_grid_column_span(column_span)
    .into()
}

pub fn variable_sized_wrap_grid_page(_: &(), _cx: &mut RenderCx) -> Element {
    let horizontal = variable_sized_wrap_grid()
        .item_width(90.0)
        .item_height(60.0)
        .width(460.0)
        .children(vec![
            tile("1", 26, 115, 232, 1, 1),
            tile("2", 46, 125, 50, 1, 1),
            tile("3", 255, 87, 34, 1, 1),
            tile("4", 121, 85, 72, 1, 1),
            tile("5", 155, 79, 150, 2, 2),
            tile("6", 0, 137, 123, 1, 1),
            tile("7", 255, 193, 7, 1, 2),
            tile("8", 96, 125, 139, 1, 1),
            tile("9", 158, 158, 158, 2, 1),
            tile("10", 69, 90, 100, 1, 1),
            tile("11", 192, 57, 43, 1, 1),
            tile("12", 103, 58, 183, 1, 1),
        ]);

    let vertical = variable_sized_wrap_grid()
        .item_width(90.0)
        .item_height(50.0)
        .orientation(Orientation::Vertical)
        .maximum_rows_or_columns(3)
        .height(280.0)
        .children(vec![
            tile("A", 26, 115, 232, 1, 1),
            tile("B", 46, 125, 50, 1, 1),
            tile("C", 255, 87, 34, 1, 1),
            tile("D", 121, 85, 72, 1, 1),
            tile("E", 155, 79, 150, 1, 1),
            tile("F", 0, 137, 123, 1, 1),
            tile("G", 255, 193, 7, 1, 1),
            tile("H", 96, 125, 139, 1, 1),
            tile("I", 158, 158, 158, 1, 1),
            tile("J", 69, 90, 100, 1, 1),
        ]);

    page_content(
        "VariableSizedWrapGrid",
        "A grid layout where items can span multiple rows and columns.",
        vec![
            sample_card(
                "Horizontal with Row/Column Spans",
                vstack((
                    text_block(
                        "Items flow left-to-right and wrap. Tile 5 spans 2x2, tile 7 spans 2 columns.",
                    )
                    .opacity(0.6)
                    .font_size(13.0),
                    horizontal,
                ))
                .spacing(8.0),
                r#"variable_sized_wrap_grid()
    .item_width(90.0).item_height(60.0)
    .children(vec![
        tile("5", ...).wrap_grid_row_span(2).wrap_grid_column_span(2),
        tile("7", ...).wrap_grid_column_span(2),
    ])"#,
            ),
            sample_card(
                "Vertical with Max Rows",
                vstack((
                    text_block(
                        "Vertical orientation fills columns first; MaximumRowsOrColumns caps the row count.",
                    )
                    .opacity(0.6)
                    .font_size(13.0),
                    vertical,
                ))
                .spacing(8.0),
                r#"variable_sized_wrap_grid()
    .orientation(Orientation::Vertical)
    .maximum_rows_or_columns(3)"#,
            ),
        ],
    )
}
