use crate::controls::*;
use windows_reactor::*;

pub fn map_control_page(_: &(), cx: &mut RenderCx) -> Element {
    let (token, set_token) = cx.use_state(String::new());
    let (status, set_status) = cx.use_state(String::from(
        "Enter a MapServiceToken to load map tiles. Without a token the control renders with the marker layer but no tile imagery.",
    ));

    let map = map_control()
        .center(-30.034647, -51.217659) // Porto Alegre, matches the official sample pin
        .zoom_level(4.0)
        .map_service_token(token.clone())
        .height(320.0)
        .width(560.0);

    let apply_token = move || {
        if token.is_empty() {
            set_status.call("Token is empty; tiles will not load.".into());
        } else {
            set_status.call("Token applied to the map control.".into());
        }
    };

    page_content(
        "Map Control",
        "Display maps and location data with MapControl, MapIcon, and MapElementsLayer.",
        vec![
            sample_card(
                "Map with Marker",
                vstack((
                    map,
                    hstack((
                        PasswordBox::new()
                            .header("Map service token")
                            .placeholder_text("Paste your Azure Maps key")
                            .width(320.0)
                            .on_password_changed(move |v| set_token.call(v)),
                        button("Apply token").on_click(apply_token),
                    ))
                    .spacing(8.0),
                    text_block(status).opacity(0.6),
                ))
                .spacing(8.0),
                r#"map_control()
    .center(-30.034647, -51.217659)
    .zoom_level(4.0)
    .map_service_token(token)
// backend: Geopoint::Create(position) -> SetCenter
//          MapIcon + MapElementsLayer -> Layers"#,
            ),
            sample_card(
                "How to get a token",
                text_block(
                    "MapControl requires a MapServiceToken from the Azure Maps account portal (learn.microsoft.com/azure/azure-maps/how-to-manage-account-keys). The control and marker layer render regardless; tile imagery appears once a valid token is set.",
                )
                .font_size(13.0),
                r#"map1.MapServiceToken = "<azure-maps-key>";"#,
            ),
        ],
    )
}
