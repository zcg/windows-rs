use crate::controls::*;
use windows_reactor::*;

pub fn line_page(_: &(), _cx: &mut RenderCx) -> Element {
    page_content(
        "Line",
        "A Line draws a straight line between two points, with configurable stroke and thickness.",
        vec![
            sample_card(
                "Basic Line",
                Shape::line(0.0, 0.0, 220.0, 40.0)
                    .stroke_rgb(0, 0, 0)
                    .stroke_thickness(2.0)
                    .width(220.0)
                    .height(40.0),
                r#"Shape::line(0.0, 0.0, 220.0, 40.0).stroke_rgb(0, 0, 0).stroke_thickness(2.0)"#,
            ),
            sample_card(
                "Thick Colored Line",
                Shape::line(0.0, 20.0, 220.0, 20.0)
                    .stroke_rgb(255, 99, 71)
                    .stroke_thickness(8.0)
                    .width(220.0)
                    .height(40.0),
                r#"Shape::line(0.0, 20.0, 220.0, 20.0).stroke_rgb(255, 99, 71).stroke_thickness(8.0)"#,
            ),
            sample_card(
                "Multiple Lines (approximating Polyline)",
                vstack((
                    Shape::line(0.0, 0.0, 110.0, 40.0).stroke_rgb(0, 0, 0).stroke_thickness(3.0).width(220.0).height(40.0),
                    Shape::line(110.0, 0.0, 220.0, 40.0).stroke_rgb(0, 0, 0).stroke_thickness(3.0).width(220.0).height(40.0),
                ))
                .spacing(0.0),
                r#"multiple Shape::line segments approximate a Polyline"#,
            ),
        ],
    )
}