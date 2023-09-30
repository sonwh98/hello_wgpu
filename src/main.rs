use std::collections::HashMap;
use std::iter;

use wgpu::util::DeviceExt;
use winit::event_loop::ControlFlow;

use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode};

use bytemuck::Pod;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

async fn init(
    window: &winit::window::Window,
) -> (
    wgpu::Instance,
    wgpu::Adapter,
    wgpu::Device,
    wgpu::Queue,
    wgpu::Surface,
    wgpu::Buffer,
    wgpu::RenderPipeline,
) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let size = window.inner_size();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let vertices: HashMap<&str, Vertex> = maplit::hashmap! {
        "A" => Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] },
        "B" => Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] },
        "C" => Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] },
        "D" => Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] },
        "E" => Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] },
    };

    let shape: &[Vertex] = &[
        *vertices.get("A").unwrap(),
        *vertices.get("B").unwrap(),
        *vertices.get("E").unwrap(),
        *vertices.get("B").unwrap(),
        *vertices.get("C").unwrap(),
        *vertices.get("E").unwrap(),
        // *vertices.get("E").unwrap(),
        // *vertices.get("C").unwrap(),
        // *vertices.get("D").unwrap()

        // *vertices.get("A").unwrap(),
        // *vertices.get("C").unwrap(),
        // *vertices.get("D").unwrap()
    ];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(shape),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let surface_caps = surface.get_capabilities(&adapter);
    // Shader code in this tutorial assumes an Srgb surface texture. Using a different
    // one will result all the colors comming out darker. If you want to support non
    // Srgb surfaces, you'll need to account for that when drawing to the frame.
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
            // or Features::POLYGON_MODE_POINT
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    });

    (
        instance,
        adapter,
        device,
        queue,
        surface,
        vertex_buffer,
        render_pipeline,
    )
}

fn render(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    surface: &wgpu::Surface,
    vertex_buffer: &wgpu::Buffer,
    render_pipeline: &wgpu::RenderPipeline,
) -> Result<(), wgpu::SurfaceError> {
    let output = surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }

    queue.submit(iter::once(encoder.finish()));
    output.present();

    Ok(())
}

async fn run() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let (instance, adapter, device, queue, surface, vertex_buffer, render_pipeline) =
        init(&window).await;

    event_loop.run(move |event, _, control_flow| {
        println!("event={:?}", event);

        match event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    winit::event::WindowEvent::CloseRequested
                    | winit::event::WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    winit::event::WindowEvent::Resized(physical_size) => {
                        //state.resize(*physical_size);
                        println!("foo1");
                    }
                    winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        println!("foo2");
                        // new_inner_size is &mut so w have to dereference it twice
                        //state.resize(**new_inner_size);
                        //render();
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                render(&device, &queue, &surface, &vertex_buffer, &render_pipeline);
                // match state.render() {
                //     Ok(_) => {}
                //     // Reconfigure the surface if it's lost or outdated
                //     Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                //         state.resize(state.size)
                //     }
                //     // The system is out of memory, we should probably quit
                //     Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                //     // We're ignoring timeouts
                //     Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                // }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn main() {
    pollster::block_on(run());
}
