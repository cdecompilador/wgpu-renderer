use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit::event::*;
use anyhow::*;

/// The state of a wgpu rendering context for a certain window
struct State {
    /// A surface is a target where to draw
    surface: wgpu::Surface,

    /// The details on how to draw a `self.surface`, contains things like the
    /// format, the size, and when the changes are propagated from the gpu to
    /// the render target (PresentMode)
    config: wgpu::SurfaceConfiguration,

    /// A connection to a logical rendering device, can interact with resources
    /// on the gpu like buffer, textures, setup the pipeline (shader modules,
    /// samplers, bind groups) and create an encoder that translates each draw
    /// command to something each associated physical device can understand
    device: wgpu::Device,

    /// Groups the translated commands to a single one and send it to the
    /// physical device
    queue: wgpu::Queue,

    /// The width of the window
    width: u32,

    /// The height of the window
    height: u32
}

impl State {
    async fn new(window: &Window) -> Self {
        todo!()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        todo!()
    }

    fn input(&mut self, event: &WindowEvent) {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn render(&mut self) -> Result<()> {
        todo!()
    }
}

fn main() -> Result<()> {
    // Initialize logging backend
    env_logger::init();

    // Create the window and the event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .context("Failed to create window")?;

    // Main loop
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            ..
        } => match event {
            // Handle exit
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    });

    Ok(())
}
