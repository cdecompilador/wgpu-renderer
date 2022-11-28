use std::path::Path;
use std::time::Instant;

use anyhow::*;
use camera::CameraController;
use cgmath::Vector3;
use model::ModelUniform;
use wgpu::TextureView;
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
use crate::uniform::{UniformGroup, UniformGroupBuilder};

type Position = cgmath::Vector3<f32>;

/// Renderer for a specific model, can render that model at multiple places
pub struct ModelRenderer {
    /// The specific model
    model: Option<Model>,

    model_uniform: ModelUniform,

    /// The instances of those models, with a fixed position
    instances: Vec<Position>,
}

impl ModelRenderer {
    /// Create the model renderer, that renders a certain model
    pub fn new(model_uniform: ModelUniform, model: Model) -> Self {
        Self {
            model: Some(model),
            model_uniform,
            instances: vec![Position::new(0.0, 0.0, 0.0)],
        }
    }

    pub fn replace_model(&mut self, new_model: Model) {
        self.model.replace(new_model);
    }

    pub fn add_instance(&mut self, position: Position) {
        self.instances.push(position);
    }

    pub fn render<'a>(
        &'a mut self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>
    ) {
        let model = self.model.as_ref().unwrap();
        for instance in &self.instances {
            self.model_uniform.update(queue, instance.clone());
            model.render(queue, render_pass);
        }
    }
}

pub struct MasterRenderer {
    /// Color used to clear the screen
    clear_color: wgpu::Color,

    /// Attached to a render pass is the programable process the data to be
    /// drawn goes through
    pipeline: ModelPipeline,

    m1: Model,
    m2: Model,
}

impl MasterRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<Self> {

        Ok(Self {
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.4,
                a: 1.0
            },
            pipeline: ModelPipeline::new(
                &device,
                format,
            )?,
            m1: Model::new(device, Mesh::QUAD),
            m2: Model::new(device, Mesh::TRIANGLE)
        })
    }

    pub fn render<'a>(
        &'a mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &TextureView
    ) {
        
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(
                        self.clear_color()
                    ),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.pipeline.set_current(&mut render_pass);
        self.m2.render(queue, &mut render_pass);
        drop(render_pass);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.pipeline.set_current(&mut render_pass);
        self.m1.render(queue, &mut render_pass);
        drop(render_pass);
    }

    pub fn clear_color(&self) -> wgpu::Color {
        self.clear_color
    }
    
    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, camera: &Camera) {
        self.pipeline.update_uniforms(queue, camera);
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

    master_renderer: MasterRenderer,
}

impl WgpuContext {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat
    ) -> Result<Self> {
        // Create the render pipeline and the uniforms, the camera must be 
        // owned by the pipeline, with the others we can do whatever we want

        let master_renderer = MasterRenderer::new(&device, format)?;

        Ok(Self {
            device,
            queue,
            master_renderer
        })
    }

    pub fn configure_surface(
        &self,
        surface: &wgpu::Surface,
        config: &wgpu::SurfaceConfiguration
    ) {
        surface.configure(&self.device, config);
    }
    
    pub fn render<'a>(&'a mut self, view: &'a wgpu::TextureView) -> Result<()> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.master_renderer.render(&self.queue, &mut encoder, view);

        // self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub fn update(&mut self, camera: &Camera) {
        self.master_renderer.update_uniforms(&self.queue, camera);
    }
}

pub struct ModelPipeline {
    pipeline: Pipeline,
    camera_uniform: CameraUniform,
    model_uniform: ModelUniform,
}

impl ModelPipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<Self> {
        // Create the shader module
        let shader = device.create_shader_module(
            wgpu::include_wgsl!("shader.wgsl")
        );

        // Create the uniform group and the uniforms
        let mut builder = UniformGroupBuilder::new(&device);
        let camera_uniform = CameraUniform::from(
            builder.create_uniform(wgpu::ShaderStages::VERTEX)
        );
        let model_uniform = ModelUniform::from(
            builder.create_uniform(wgpu::ShaderStages::VERTEX)
        );
        let uniform_group = builder.build();

        Ok(Self {
            pipeline: Pipeline::new(device, format, uniform_group, shader)?,
            camera_uniform,
            model_uniform
        })
    }

    pub fn set_current<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.pipeline.set_current(render_pass);
    }

    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, camera: &Camera) {
        self.camera_uniform.update_view_proj(queue, camera);
    }
}

pub struct Pipeline {
    shader: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,
    uniform_group: UniformGroup,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        uniform_group: UniformGroup,
        shader: wgpu::ShaderModule,
    ) -> Result<Self> {
        // Create the pipeline and add its bind groups, the shader must have
        // fixed entry point names
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    uniform_group.bind_group_layout(),
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
            uniform_group,
        })
    }
    
    pub fn set_current<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.uniform_group.bind_group(), &[]);
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