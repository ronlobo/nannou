//! The lower-level "raw" frame type allowing to draw directly to the window's swap chain image.

use crate::geom;
use crate::wgpu;
use crate::window;
use std::sync::Mutex;

/// Allows the user to draw a single **RawFrame** to the surface of a window.
///
/// The application's **view** function is called each time the application is ready to retrieve a
/// new image that will be displayed to a window. The **RawFrame** type can be thought of as the
/// canvas to which you draw this image.
///
/// ## Under the hood - WGPU
///
/// **RawFrame** provides access to the **wgpu::TextureView** associated with the swap chain's
/// current target texture for a single window.
///
/// In the case that your **view** function is shared between multiple windows, can determine which
/// window the **RawFrame** is associated with via the **RawFrame::window_id** method.
///
/// The user can draw to the swap chain texture by building a list of commands via a
/// `wgpu::CommandEncoder` and submitting them to the `wgpu::Queue` associated with the
/// `wgpu::Device` that was used to create the swap chain. It is important that the queue
/// matches the device. In an effort to reduce the chance for errors to occur, **RawFrame**
/// provides access to a `wgpu::CommandEncoder` whose commands are guaranteed to be submitted to
/// the correct `wgpu::Queue` at the end of the **view** function.
pub struct RawFrame<'swap_chain> {
    command_encoder: Mutex<wgpu::CommandEncoder>,
    window_id: window::Id,
    nth: u64,
    swap_chain_texture: &'swap_chain wgpu::TextureView,
    queue: &'swap_chain wgpu::Queue,
    texture_format: wgpu::TextureFormat,
    window_rect: geom::Rect,
}

impl<'swap_chain> RawFrame<'swap_chain> {
    // Initialise a new empty frame ready for "drawing".
    pub(crate) fn new_empty(
        device: &'swap_chain wgpu::Device,
        queue: &'swap_chain wgpu::Queue,
        window_id: window::Id,
        nth: u64,
        swap_chain_texture: &'swap_chain wgpu::TextureView,
        texture_format: wgpu::TextureFormat,
        window_rect: geom::Rect,
    ) -> Self {
        let ce_desc = wgpu::CommandEncoderDescriptor::default();
        let command_encoder = device.create_command_encoder(&ce_desc);
        let command_encoder = Mutex::new(command_encoder);
        let frame = RawFrame {
            command_encoder,
            window_id,
            nth,
            swap_chain_texture,
            queue,
            texture_format,
            window_rect,
        };
        frame
    }

    // Called after the user's `view` function, this consumes the `RawFrame` and returns the inner
    // command buffer builder so that it can be completed.
    pub(crate) fn finish(self) -> wgpu::CommandEncoder {
        let RawFrame {
            command_encoder, ..
        } = self;
        command_encoder
            .into_inner()
            .expect("failed to lock `command_encoder`")
    }

    /// Access the command encoder in order to encode commands that will be submitted to the swap
    /// chain queue at the end of the call to **view**.
    pub fn command_encoder(&self) -> std::sync::MutexGuard<wgpu::CommandEncoder> {
        self.command_encoder
            .lock()
            .expect("failed to acquire lock to command encoder")
    }

    /// The `Id` of the window whose vulkan surface is associated with this frame.
    pub fn window_id(&self) -> window::Id {
        self.window_id
    }

    /// A **Rect** representing the full surface of the frame.
    ///
    /// The returned **Rect** is equivalent to the result of calling **Window::rect** on the window
    /// associated with this **Frame**.
    pub fn rect(&self) -> geom::Rect {
        self.window_rect
    }

    /// The `nth` frame for the associated window since the application started.
    ///
    /// E.g. the first frame yielded will return `0`, the second will return `1`, and so on.
    pub fn nth(&self) -> u64 {
        self.nth
    }

    /// The swap chain texture that will be the target for drawing this frame.
    pub fn swap_chain_texture(&self) -> &wgpu::TextureView {
        &self.swap_chain_texture
    }

    /// The texture format of the frame's swap chain texture.
    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.texture_format
    }

    /// The queue on which the swap chain was created and which will be used to submit the
    /// **RawFrame**'s encoded commands.
    pub fn queue(&self) -> &wgpu::Queue {
        self.queue
    }
}
