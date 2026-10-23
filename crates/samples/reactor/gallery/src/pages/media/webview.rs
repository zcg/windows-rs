use crate::controls::*;
use windows_reactor::*;
use windows_webview::{EventRegistration, ProcessFailedArgs, ProcessFailedKind, WebView, webview};

const HOME: &str = "https://learn.microsoft.com/windows/apps/";

pub struct WebViewPage {
    address: String,
    status: String,
    web: Option<WebView>,
    registrations: Vec<EventRegistration>,
}

#[derive(Clone)]
pub enum Message {
    Ready(WebView),
    AddressChanged(String),
    Navigate,
    Back,
    Forward,
    Reload,
    Navigated(String),
    ProcessExited,
    Failed(String),
}

impl Component for WebViewPage {
    type Message = Message;
    type Input = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self {
            address: String::from(HOME),
            status: String::from("Loading..."),
            web: None,
            registrations: Vec::new(),
        }
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Ready(web) => {
                let sender = context.sender();
                self.registrations.clear();
                let reflect = web.clone();
                let navigate_sender = sender.clone();
                if let Ok(registration) = web.on_navigation_completed(move |_| {
                    _ = navigate_sender.send(Message::Navigated(reflect.source()));
                }) {
                    self.registrations.push(registration);
                }
                if let Ok(registration) = web.on_process_failed(move |args: ProcessFailedArgs| {
                    if matches!(args.kind(), ProcessFailedKind::BrowserProcessExited) {
                        _ = sender.send(Message::ProcessExited);
                    } else {
                        _ = sender.send(Message::Failed(format!(
                            "WebView process failed: {:?}",
                            args.kind()
                        )));
                    }
                }) {
                    self.registrations.push(registration);
                }
                self.status = format!("Loading {HOME}...");
                if web.navigate(HOME).is_err() {
                    self.status = format!("Could not navigate to {HOME}");
                }
                self.web = Some(web);
            }
            Message::AddressChanged(address) => {
                self.address = address;
            }
            Message::Navigate => {
                if let Some(web) = self.web.as_ref() {
                    let url = normalize(&self.address);
                    if web.navigate(&url).is_err() {
                        self.status = format!("Could not navigate to {url}");
                    } else {
                        self.status = format!("Loading {url}...");
                    }
                    self.address = url;
                }
            }
            Message::Back => {
                if let Some(web) = self.web.as_ref() {
                    _ = web.go_back();
                }
            }
            Message::Forward => {
                if let Some(web) = self.web.as_ref() {
                    _ = web.go_forward();
                }
            }
            Message::Reload => {
                if let Some(web) = self.web.as_ref() {
                    _ = web.reload();
                }
            }
            Message::Navigated(source) => {
                if !source.is_empty() {
                    self.address = source;
                }
                self.status = format!("Loaded {}", self.address);
            }
            Message::ProcessExited => {
                self.web = None;
                self.registrations.clear();
                self.status = String::from("Browser process exited.");
            }
            Message::Failed(status) => {
                self.status = status;
            }
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        let toolbar = Grid::new()
            .columns([
                GridLength::Auto,
                GridLength::Auto,
                GridLength::Auto,
                GridLength::STAR,
                GridLength::Auto,
            ])
            .column_spacing(8.0)
            .grid_row(0)
            .children((
                Button::new()
                    .on_click(context.message(Message::Back))
                    .grid_column(0)
                    .content("←"),
                Button::new()
                    .on_click(context.message(Message::Forward))
                    .grid_column(1)
                    .content("→"),
                Button::new()
                    .on_click(context.message(Message::Reload))
                    .grid_column(2)
                    .content("↻"),
                TextBox::new()
                    .text(self.address.clone())
                    .placeholder_text("Enter a URL")
                    .on_text_changed(context.callback(|value: std::rc::Rc<str>| {
                        Message::AddressChanged(value.to_string())
                    }))
                    .grid_column(3),
                Button::new()
                    .on_click(context.message(Message::Navigate))
                    .grid_column(4)
                    .content("Go"),
            ));
        let browser = Grid::new()
            .rows([GridLength::Auto, GridLength::STAR])
            .children((
                toolbar,
                Border::new()
                    .height(500.0)
                    .grid_row(1)
                    .content(webview(context.callback(Message::Ready))),
            ));
        page_content(
            "WebView2",
            "A Microsoft Edge WebView2 control embedded in a Reactor app.",
            [KeyedView::new(
                "minimal-browser",
                sample_card(
                    "Minimal Browser",
                    StackPanel::new().spacing(8.0).children((
                        browser,
                        TextBlock::new().text(self.status.clone()).opacity(0.6),
                    )),
                ),
            )],
        )
    }
}

fn normalize(address: &str) -> String {
    let address = address.trim();
    if address.contains("://") {
        address.to_string()
    } else {
        format!("https://{address}")
    }
}
