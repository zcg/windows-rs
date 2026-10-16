use crate::controls::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use windows_canvas::{Brush, ColorF, DrawContext, Ellipse, animated_canvas};
use windows_numerics::{Matrix3x2, Vector2};
use windows_reactor::*;

pub struct SwapChainPanelPage {
    start: Instant,
    brushes: Rc<RefCell<Option<Brushes>>>,
}

/// Device resources for the radar drawing, rebuilt when the surface reports a new device.
struct Brushes {
    ring: Brush,
    wedge: Brush,
    center: Brush,
}

impl Component for SwapChainPanelPage {
    type Message = ();
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            start: Instant::now(),
            brushes: Rc::new(RefCell::new(None)),
        }
    }

    fn view(&self, _: &(), _: &mut ViewContext<Self>) -> View {
        let start = self.start;
        let brushes = Rc::clone(&self.brushes);
        let surface = animated_canvas(move |ctx| {
            if ctx.device_changed() || brushes.borrow().is_none() {
                *brushes.borrow_mut() = Some(Brushes {
                    ring: ctx.create_solid_brush(ColorF::new(0.38, 0.62, 0.92, 0.45))?,
                    wedge: ctx.create_solid_brush(ColorF::new(0.58, 0.86, 1.0, 0.95))?,
                    center: ctx.create_solid_brush(ColorF::new(0.92, 0.96, 1.0, 1.0))?,
                });
            }
            let brushes = brushes.borrow();
            draw_radar(
                ctx,
                start.elapsed().as_secs_f32(),
                brushes.as_ref().unwrap(),
            );
            Ok(())
        });

        page_content(
            "SwapChainPanel",
            "Hosts a Direct2D swap chain inside the XAML tree.",
            [
                KeyedView::new(
                    "surface",
                    sample_card(
                        "Animated Direct2D surface",
                        Border::new().height(280.0).content(surface),
                    ),
                ),
                KeyedView::new(
                    "binding",
                    sample_card(
                        "How the panel is bound",
                        InfoBar::new()
                            .severity(InfoBarSeverity::Informational)
                            .title("XAML hosts the swap chain, Canvas draws into it")
                            .message(
                                "SwapChainPanel::new().element_ref(&panel) publishes the XAML \
                                 element, observe_surface reports its size and composition \
                                 frames, and window-canvas creates the DXGI swap chain through \
                                 ISwapChainPanelNative::SetSwapChain. Each rendering event \
                                 redraws the surface; resizing or moving the panel to another \
                                 monitor rebuilds the chain at the new scale.",
                            )
                            .is_open(true),
                    ),
                ),
            ],
        )
    }
}

/// Draws a sweeping radar dial sized to the current surface.
fn draw_radar(ctx: &DrawContext<'_>, seconds: f32, brushes: &Brushes) {
    ctx.clear(ColorF::new(0.055, 0.075, 0.11, 1.0));

    let center = Vector2::new(ctx.width / 2.0, ctx.height / 2.0);
    let radius = (ctx.width.min(ctx.height) / 2.0 - 16.0).max(24.0);

    for step in 1..=3 {
        let ring = Ellipse::circle(center, radius * step as f32 / 3.0);
        ctx.draw_ellipse(&ring, &brushes.ring, 1.0);
    }

    let sweep = (seconds * 72.0) % 360.0;
    let origin = Matrix3x2::translation(center.x, center.y);
    ctx.with_transform(&(Matrix3x2::rotation(sweep) * origin), || {
        ctx.draw_line(
            Vector2::zero(),
            Vector2::new(0.0, -radius),
            &brushes.wedge,
            2.0,
        );
    });

    ctx.fill_ellipse(&Ellipse::circle(center, 4.0), &brushes.center);
}
