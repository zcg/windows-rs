use crate::controls::*;
use windows_reactor::*;

pub fn shape_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Shape",
        "Shapes are the base class for drawing 2D geometry: Rectangle, Ellipse, and Line.",
        vec![
            sample_card(
                "Ellipse",
                vstack((
                    Shape::ellipse()
                        .fill_rgb(100, 149, 237)
                        .width(120.0)
                        .height(80.0),
                    Shape::ellipse()
                        .stroke_rgb(0, 0, 0)
                        .stroke_thickness(2.0)
                        .width(120.0)
                        .height(80.0),
                ))
                .spacing(12.0),
                r#"Shape::ellipse().fill_rgb(100, 149, 237).width(120.0).height(80.0)"#,
            ),
            sample_card(
                "Rectangle",
                vstack((
                    Shape::rectangle()
                        .fill_rgb(255, 99, 71)
                        .width(160.0)
                        .height(90.0),
                    Shape::rectangle()
                        .fill_rgb(144, 238, 144)
                        .corner_radius(12.0)
                        .width(160.0)
                        .height(90.0),
                ))
                .spacing(12.0),
                r#"Shape::rectangle().corner_radius(12.0).fill_rgb(144, 238, 144)"#,
            ),
            sample_card(
                "Line",
                Shape::line(0.0, 0.0, 200.0, 60.0)
                    .stroke_rgb(0, 0, 0)
                    .stroke_thickness(4.0)
                    .width(200.0)
                    .height(60.0),
                r#"Shape::line(0.0, 0.0, 200.0, 60.0).stroke_rgb(0, 0, 0).stroke_thickness(4.0)"#,
            ),
        ],
    )
}
