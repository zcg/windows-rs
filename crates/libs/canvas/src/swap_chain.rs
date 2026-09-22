use super::*;

/// Manages a DXGI swap chain.
pub struct SwapChain {
    swap_chain: IDXGISwapChain1,
    d2d_context: ID2D1DeviceContext,
    device_lost_flag: Cell<bool>,
    width: u32,
    height: u32,
    dpi_x: f32,
    dpi_y: f32,
}

impl SwapChain {
    pub(crate) fn new(device: &GpuDevice, width: u32, height: u32) -> Result<Self> {
        let desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
            ..Default::default()
        };

        let swap_chain = unsafe {
            device
                .dxgi_factory()
                .CreateSwapChainForComposition(device.d3d_device(), &desc, None)?
        };

        Self::from_swap_chain(device, swap_chain, width, height)
    }

    pub(crate) fn new_for_hwnd(
        device: &GpuDevice,
        hwnd: *mut core::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            ..Default::default()
        };

        let swap_chain = unsafe {
            device.dxgi_factory().CreateSwapChainForHwnd(
                device.d3d_device(),
                hwnd,
                &desc,
                None,
                None,
            )?
        };

        Self::from_swap_chain(device, swap_chain, width, height)
    }

    fn from_swap_chain(
        device: &GpuDevice,
        swap_chain: IDXGISwapChain1,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let d2d_context = unsafe {
            device
                .d2d_device()
                .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?
        };

        let mut result = Self {
            swap_chain,
            d2d_context,
            device_lost_flag: Cell::new(false),
            width,
            height,
            dpi_x: 96.0,
            dpi_y: 96.0,
        };
        result.set_target()?;
        Ok(result)
    }

    /// Resizes the swap chain buffers. A zero width or height is ignored.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            return Ok(());
        }
        unsafe {
            self.d2d_context.SetTarget(None);
            self.swap_chain
                .ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, 0)
                .ok()?;
        }
        self.width = width;
        self.height = height;
        self.set_target()
    }

    /// Begins drawing a frame, returning a [`DrawingSession`].
    pub fn begin_draw(&mut self) -> Result<DrawingSession<'_>> {
        self.device_lost_flag.set(false);
        DrawingSession::new(&self.d2d_context, &self.device_lost_flag)
    }

    /// Presents the frame just drawn.
    ///
    /// Returns `Ok(false)` if the device was lost.
    ///
    /// # 为什么不在这里等 vblank（曾经的缺陷）
    ///
    /// 调用点是 [`CanvasRuntime::render_frame`](crate::reactor::CanvasRuntime)，
    /// 而它由 WinUI 的 `CompositionTarget::Rendering` 合成回调驱动 —— 那是 **UI
    /// 线程**，与 `WM_MOVING` / `WM_SIZING` / `WM_NCLBUTTONDOWN` 和全部鼠标输入
    /// **共用同一个线程**。旧实现是 `Present(1, 0)`（sync interval = 1），即
    /// **同步等待下一个 vblank 才返回**：
    ///
    /// * 60Hz 屏：每帧在 UI 线程上白等 ≈16.7ms；
    /// * 120Hz 屏：每帧白等 ≈8.3ms。
    ///
    /// 于是一秒的帧预算被 present 吃掉 100%（按刷新率），窗口拖拽、边框缩放、
    /// 最大化/最小化都只能挤在 vblank 之间的缝隙里处理 → 直接表现为"窗口操作
    /// 不跟手"。画布应用帧率越高，这个缺陷越明显。
    ///
    /// 现在改为 `Present(0, DXGI_PRESENT_DO_NOT_WAIT)`：DXGI 队列仍被上一帧
    /// 占用时**立即**返回 `DXGI_ERROR_WAS_STILL_DRAWING`，本帧丢弃而不是等待。
    ///
    /// 丢帧**不会丢内容**：
    /// 1. 交换链保持上一帧，屏幕上不会闪黑或回退；
    /// 2. 合成回调本来就按显示器刷新率持续驱动下一帧，下一个 vblank 后的第一
    ///    次 `Rendering` 会把最新内容再画一遍。
    ///
    /// 因此这个丢弃是**有意的节流**，返回值按"成功"处理（`Ok(true)`）——它不是
    /// 错误，调用方（尤其是 `Canvas::animated`）不需要、也不应该把它当失败重试。
    pub fn present(&self) -> Result<bool> {
        if self.device_lost_flag.get() {
            return Ok(false);
        }
        let result = unsafe { self.swap_chain.Present(0, DXGI_PRESENT_DO_NOT_WAIT) };
        if is_device_lost(result) {
            return Ok(false);
        }
        if result == DXGI_ERROR_WAS_STILL_DRAWING {
            // 队列忙：本帧丢弃。见上方文档——这是设计行为，不是失败。
            return Ok(true);
        }
        result.ok().map(|()| true)
    }

    pub fn create_solid_brush(&self, color: ColorF) -> Result<Brush> {
        let c: D2D_COLOR_F = color.into();
        unsafe { self.d2d_context.CreateSolidColorBrush(&c, None).map(Brush) }
    }

    pub fn load_bitmap(&self, path: impl AsRef<std::path::Path>) -> Result<Bitmap> {
        Bitmap::load_from_file(&self.d2d_context, path.as_ref())
    }

    pub fn raw_swap_chain(&self) -> &IDXGISwapChain1 {
        &self.swap_chain
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Sets the Direct2D DPI for subsequent rendering.
    pub fn set_dpi(&mut self, dpi_x: f32, dpi_y: f32) {
        self.dpi_x = dpi_x;
        self.dpi_y = dpi_y;
        unsafe { self.d2d_context.SetDpi(dpi_x, dpi_y) }
        let _ = self.set_target();
    }

    /// Applies an inverse composition scale for high-DPI presentation.
    pub fn set_composition_scale(&self, scale_x: f32, scale_y: f32) {
        if let Ok(sc2) = self.swap_chain.cast::<IDXGISwapChain2>() {
            let matrix = DXGI_MATRIX_3X2_F {
                _11: 1.0 / scale_x,
                _12: 0.0,
                _21: 0.0,
                _22: 1.0 / scale_y,
                _31: 0.0,
                _32: 0.0,
            };
            unsafe {
                let _ = sc2.SetMatrixTransform(&matrix);
            }
        }
    }

    /// Returns `true` if the device was lost during the last frame.
    pub fn is_device_lost(&self) -> bool {
        self.device_lost_flag.get()
    }

    fn set_target(&mut self) -> Result<()> {
        unsafe {
            let surface: IDXGISurface = self.swap_chain.GetBuffer(0)?;
            let props = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: self.dpi_x,
                dpiY: self.dpi_y,
                bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                ..Default::default()
            };
            let bitmap = self
                .d2d_context
                .CreateBitmapFromDxgiSurface(&surface, Some(&props))?;
            self.d2d_context.SetTarget(&bitmap);
            self.d2d_context.SetDpi(self.dpi_x, self.dpi_y);
            Ok(())
        }
    }
}
