use std::mem;
use std::path::Path;
use std::time::Instant;

use anyhow::*;
use camera::CameraController;
use mouse_input::MouseInput;
use winit::dpi::PhysicalSize;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

mod camera;
mod mesh;
mod model;
mod uniform;
mod mouse_input;

use crate::camera::{Camera, CameraUniform};
use crate::mesh::{Mesh, VERTEX_DESC};
use crate::model::Model;

type Position = cgmath::Vector3<f32>;

/// Renderer for a specific model, can render that model at multiple places
pub struct ModelRenderer<'a> {
    /// The specific model
    model: Option<Model<'a>>,

    /// The instances of those models, with a fixed position
    instances: Vec<Position>,
}

impl<'a> ModelRenderer<'a> {
    /// Create the model renderer, that renders a certain model
    pub fn new(model: Model<'a>) -> Self {
        Self {
            model: Some(model),
            instances: vec![Position::new(0.0, 0.0, 0.0)],
        }
    }

    pub fn replace_model(&mut self, new_model: Model<'a>) {
        self.model.replace(new_model);
    }

    pub fn add_instance(&mut self, position: Position) {
        self.instances.push(position);
    }

    pub fn render(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        let mut model = self.model.as_mut().unwrap();
        for instance in &self.instances {
            // model.set_instance(instance);
            model.render(render_pass);
        }
    }
}

pub struct MasterRenderer<'a> {
    model_renderer: ModelRenderer<'a>,
    clear_color: wgpu::Color,
}

impl<'a> MasterRenderer<'a> {
    pub fn new(context: &WgpuContext) -> Self {
        Self {
            model_renderer: ModelRenderer::new(
                Model::new(
                    context.device(),
                    Mesh::QUAD
                )
            ),
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.4,
                a: 1.0
            }
        }
    }
}


/// Contains all the wgpu primitives and state
pub struct WgpuContext {
    /// A connection to a logical rendering device, can interact with resources
    /// on the gpu like buffer, textures, setup the pipeline (shader modules,
    /// samplers, bind groups) and create an encoder that translated each draw
    /// command too something each associated physical device can understand
    device: wgpu::Device,

    /// Groups the translated commands to a single one and send it to the
    /// physical device
    queue: wgpu::Queue,

    /// Attached to a render pass is the programable process the data to be
    /// drawn goes through
    pipeline: Pipeline,

    /// Color used to clear the screen
    clear_color: wgpu::Color,
}

impl WgpuContext {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat
    ) -> Result<Self> {
        // Create the render pipeline
        let pipeline = Pipeline::new(&device, format, ".\\src\\shader.wgsl")?;

        Ok(Self {
            device,
            queue,
            pipeline,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        })
    }

    pub fn configure_surface(
        &self,
        surface: &wgpu::Surface,
        config: &wgpu::SurfaceConfiguration
    ) {
        surface.configure(&self.device, config);
    }
    
    pub fn render(&mut self, view: &wgpu::TextureView) -> Result<()> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let model = Model::new(&self.device, Mesh::QUAD);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.pipeline.set_current(&mut render_pass);
            model.render(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub fn update(&mut self, camera: &Camera) {
        self.pipeline.camera_uniform.update_view_proj(&self.queue, camera);
    }

    pub fn queue<'a>(&'a self) -> &'a wgpu::Queue {
        &self.queue
    }

    pub fn device<'a>(&'a self) -> &'a wgpu::Device {
        &self.device
    }
}

pub struct Pipeline {
    shader: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,
    pub camera_uniform: CameraUniform,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        shader_source_path: impl AsRef<Path>,
    ) -> Result<Self> {
        // Create the camera uniform
        let mut camera_uniform = CameraUniform::new(device);

        // Load the source from the path
        let shader_source = std::fs::read_to_string(shader_source_path.as_ref())
            .context("Failed to read the shader path")?;

        // Create the shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!(
                "Shader: {}",
                shader_source_path.as_ref().display()
            )),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create the pipeline and add its bind groups, the shader must have
        // fixed entry point names
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    camera_uniform.bind_group_layout()
                ],
                push_constant_ranges: &[],
            });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VERTEX_DESC],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(Self {
            shader,
            pipeline,
            camera_uniform
        })
    }
    
    pub fn set_current<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.camera_uniform.bind_group(), &[]);
    }
}

/// The state of a wgpu rendering context for a certain window
struct Display {
    /// A surface is a target where to draw
    surface: wgpu::Surface,

    /// The details on how to draw a `self.surface`, contains things like the
    /// format, the size, and when the changes are propagated from the gpu to
    /// the render target (PresentMode)
    config: wgpu::SurfaceConfiguration,

    context: WgpuContext,

    camera: Camera,

    camera_controller: CameraController,

    /// The width of the window
    width: u32,

    /// The height of the window
    height: u32,
}

impl Display {
    async fn new(window: &Window) -> Result<Self> {
        // Extract the size of the window
        let PhysicalSize { width, height } = window.inner_size();

        // Initialize wgpu and get a physical device compatible to the
        // surface created by the window
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // Get a logical device (default limits/features)
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: Some("My device"),
                },
                None,
            )
            .await
            .unwrap();

        // Create the surface configuration
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface
                .get_supported_formats(&adapter)
                .get(0)
                .context("No supported format available for surface")
                .unwrap(),
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        // Create the wgpu rendering context and configure the surface with that
        // config using it
        let context = WgpuContext::new(device, queue, config.format)?;
        context.configure_surface(&surface, &config);

        // Create the camera
        let camera = 
            Camera::new(width, height, (0.0, 0.0, 0.0), cgmath::Deg(90.0));

        Ok(Self {
            surface,
            config,
            context,
            camera,
            camera_controller: CameraController::new(1.0, 0.01),
            width,
            height,
        })
    }

    fn resize(&mut self, PhysicalSize { width, height }: PhysicalSize<u32>) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
            self.config.width = width;
            self.config.height = height;
            self.context.configure_surface(&self.surface, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_event(event)
    }

    fn process_device_event(&mut self, event: &DeviceEvent) {
        self.camera_controller.process_device_event(event);
    }

    fn update(&mut self, dt: f32) {
        self.context.update(&self.camera);
        self.camera_controller.update_camera(&mut self.camera, dt);
    }

    fn render(&mut self) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.context.render(&view)?;

        output.present();

        Ok(())
    }
}

fn main() -> Result<()> {
    // Initialize the logging backend
    env_logger::init();

    // Create the event loop and the window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .context("Failed to create window")?;

    // Initialize wgpu rendering context
    let mut state = pollster::block_on(Display::new(&window))?;

    // Main loop
    let mut dt = 0.0;
    let mut it = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { ref event, .. } if !state.input(event) => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::R),
                            ..
                        },
                    ..
                } => {
                    window.request_redraw();
                }
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                state.update(dt);
                if let Err(e) = state.render() {
                    if let Some(e) = e.downcast_ref::<wgpu::SurfaceError>() {
                        match e {
                            wgpu::SurfaceError::Lost => {
                                state.resize(PhysicalSize::new(state.width, state.height));
                            }
                            wgpu::SurfaceError::OutOfMemory => {
                                *control_flow = ControlFlow::Exit;
                            }
                            _ => {
                                panic!("Unknown render error: {}", e);
                            }
                        }
                    }
                }
            },
            Event::DeviceEvent { 
                ref event,
                .. 
            } => {
                state.process_device_event(event);
            }
            Event::MainEventsCleared => {
                dt = it.elapsed().as_secs_f32();
                it = Instant::now();
                window.request_redraw();
            }
            _ => {}
        }
    })
}