use env_logger;
use wgpu::{
    util::DeviceExt, Backends, Instance, PowerPreference, RequestAdapterOptions,
    SurfaceConfiguration,
};
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct State {
    zoom: f32,
    offset: [f32; 2],
    mouse_pos: [f32; 2],
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();
    let instance = Instance::new(Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(&Default::default(), None)
        .await
        .unwrap();

    let mut config = SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    // State for zoom and offset
    let mut state = State {
        zoom: 1.0,
        offset: [0.0, 0.0],
        mouse_pos: [0.0, 0.0],
    };

    // Uniform buffer for zoom and offset
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[state.zoom, state.offset[0], state.offset[1]]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Bind group and layout for uniforms
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT, // Or VERTEX if it's used in the vertex shader
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("Bind Group"),
    });

    // Modify render pipeline layout to include bind group layout
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    // Re-create render pipeline with new layout
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
    });

    fn window_to_scene_coords(
        mouse_pos: [f32; 2],
        zoom: f32,
        offset: [f32; 2],
        window_size: [f32; 2],
    ) -> [f32; 2] {
        // Convert mouse position from pixels to normalized device coordinates (-1 to 1)
        let ndc_x = (mouse_pos[0] / window_size[0]) * 2.0 - 1.0;
        let ndc_y = (mouse_pos[1] / window_size[1]) * 2.0 - 1.0;

        // Adjust for zoom and offset
        let scene_x = ndc_x * zoom - offset[0];
        let scene_y = ndc_y * zoom - offset[1];

        [scene_x, scene_y]
    }

    fn adjust_offset_for_zoom(
        scene_pos_before_zoom: [f32; 2],
        new_zoom: f32,
        old_offset: [f32; 2],
        window_size: [f32; 2],
    ) -> [f32; 2] {
        // Convert the scene position back to NDC at the new zoom level
        let ndc_x = (scene_pos_before_zoom[0] + old_offset[0]) / new_zoom;
        let ndc_y = (scene_pos_before_zoom[1] + old_offset[1]) / new_zoom;

        // Convert NDC to screen space (pixels)
        let screen_x = (ndc_x + 1.0) * 0.5 * window_size[0];
        let screen_y = (ndc_y + 1.0) * 0.5 * window_size[1];

        // Calculate the difference in screen space
        let delta_x = screen_x - scene_pos_before_zoom[0];
        let delta_y = screen_y - scene_pos_before_zoom[1];

        // Convert this difference back to scene coordinates
        let scene_delta_x = delta_x * new_zoom;
        let scene_delta_y = delta_y * new_zoom;

        // Adjust the offset by this difference
        [old_offset[0] - scene_delta_x, old_offset[1] - scene_delta_y]
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
                let output = match surface.get_current_frame() {
                    Ok(output) => output,
                    Err(wgpu::SurfaceError::Outdated) => {
                        surface.configure(&device, &config);
                        surface.get_current_frame().unwrap()
                    }
                    Err(_) => return,
                };
                let view = output
                    .output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[wgpu::RenderPassColorAttachment {
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
                        }],
                        depth_stencil_attachment: None,
                    });

                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.draw(0..6, 0..1);
                }

                queue.submit(Some(encoder.finish()));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let PhysicalPosition { x, y } = position;
                state.mouse_pos = [x as f32, y as f32];
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                // Convert mouse position to scene coordinates
                let scene_pos = window_to_scene_coords(
                    state.mouse_pos,
                    state.zoom,
                    state.offset,
                    [config.width as f32, config.height as f32],
                );

                // Adjust zoom
                state.zoom /= 1.1;
                let new_zoom = state.zoom / 1.1; // Adjust this factor based on your zooming logic

                let window_size = [config.width as f32, config.height as f32];
                // Adjust offset based on the new zoom level

                state.offset = adjust_offset_for_zoom(
                    scene_pos,
                    new_zoom, // New zoom level
                    state.offset,
                    window_size,
                );
                // Update uniform buffer with new zoom and offset values
                queue.write_buffer(
                    &uniform_buffer,
                    0,
                    bytemuck::cast_slice(&[state.zoom, state.offset[0], state.offset[1]]),
                );
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(800, 600))
        .with_title("Mandelbrot Set")
        .build(&event_loop)
        .unwrap();

    pollster::block_on(run(event_loop, window));
}
