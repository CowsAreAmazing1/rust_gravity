

use nannou::prelude::*;
use bytemuck::{Pod, Zeroable};


const WORK_GROUP_SIZE: u32 = 256;


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct QuadVertex {
    pos: [f32; 2],
}

impl QuadVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0, // Matches @location(1) in VertexInput
                format: wgpu::VertexFormat::Float32x2,
            }]
        }
    }
}

pub const QUAD_VERTICES: &[QuadVertex] = &[
    QuadVertex { pos: [-0.5, -0.5] }, // Bottom-left
    QuadVertex { pos: [ 0.5, -0.5] }, // Bottom-right
    QuadVertex { pos: [-0.5,  0.5] }, // Top-left
    QuadVertex { pos: [ 0.5,  0.5] }, // Top-right
];




#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Uniforms {
    pub scale: f32,
    aspect_ratio: f32,
    pub camera_translation: [f32; 2], // Camera drag translation
    pub window_size: [f32; 2], // [width, height] in pixels
    pub rotation_angle: f32,    // Rotation angle in radians
    _padding: f32,
    pub rotation_center: [f32; 2], // Point to rotate around (in world coordinates)
}

impl Uniforms {
    pub fn new(scale: f32, camera_translation: Vec2, window_size: Vec2) -> Self {
        Self {
            scale,
            aspect_ratio: window_size.x / window_size.y,
            camera_translation: camera_translation.into(),
            window_size: window_size.into(),
            rotation_angle: 0.0,
            _padding: 0.0,
            rotation_center: [0.0, 0.0],
        }
    }

    pub fn with_rotation(scale: f32, camera_translation: Vec2, window_size: Vec2, rotation_angle: f32, rotation_center: Vec2) -> Self {
        Self {
            scale,
            aspect_ratio: window_size.x / window_size.y,
            camera_translation: camera_translation.into(),
            window_size: window_size.into(),
            rotation_angle,
            _padding: 0.0,
            rotation_center: rotation_center.into(),
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct DispatchParams {
    offset: u32,
    dt: f32,
}



#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct GpuAttractor {
    position: [f32; 2],
    mass: f32,
    _padding: f32,
}

impl GpuAttractor {
    pub fn new(position: Vec2, mass: f32) -> Self {
        GpuAttractor {
            position: [position.x, position.y],
            mass,
            _padding: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct GpuDust {
    position: [f32; 2],
    velocity: [f32; 2],
}

impl GpuDust {
    pub fn new(position: Vec2, velocity: Vec2) -> Self {
        GpuDust {
            position: [position.x, position.y],
            velocity: [velocity.x, velocity.y],
        }
    }

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GpuDust>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1, // @location(1) particle_pos
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 2, // @location(2) particle_vel
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}


pub struct GpuState {
    compute_pipeline: wgpu::ComputePipeline,
    pub render_pipeline: wgpu::RenderPipeline,
    pub attractor_buffer: wgpu::Buffer,
    pub dust_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub dispatch_buffer: wgpu::Buffer,
    pub attractor_bind_group: wgpu::BindGroup,
    pub dust_bind_group: wgpu::BindGroup,
    pub uniform_bind_group: wgpu::BindGroup,
    pub dispatch_bind_group: wgpu::BindGroup,

    pub num_particles: u32
}

impl GpuState {
    pub fn new(device: &wgpu::Device, attractors: &[GpuAttractor], dust_particles: &[GpuDust]) -> Self {
        let attractor_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Attractor Buffer"),
            contents: bytemuck::cast_slice(attractors),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let dust_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dust Buffer"),
            contents: bytemuck::cast_slice(dust_particles),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX, // | wgpu::BufferUsages::COPY_DST,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let dispatch_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dispatch Params Buffer"),
            size: std::mem::size_of::<DispatchParams>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind groups
        let attractor_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("Attractor Bind Group Layout"),
        });

        let attractor_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &attractor_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: attractor_buffer.as_entire_binding(),
                },
            ],
            label: Some("Attractor Bind Group"),
        });

        let dust_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Dust Bind Group Layout"),
        });

        let dust_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &dust_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: dust_buffer.as_entire_binding()
                },
            ],
            label: Some("Dust Bind Group"),
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("Uniform Bind Group Layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("Uniform Bind Group"),
        });

        let dispatch_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("Dispatch Params Bind Group Layout"),
        });

        let dispatch_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &dispatch_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: dispatch_buffer.as_entire_binding(),
                },
            ],
            label: Some("Dispatch Bind Group"),
        });

        // Load shaders
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/compute.wgsl").into()),
        });

        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/render.wgsl").into()),
        });

        // Create pipeline layouts
        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&dust_bind_group_layout, &attractor_bind_group_layout, &dispatch_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipelines
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[QuadVertex::desc(), GpuDust::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: Frame::TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        GpuState { 
            compute_pipeline,
            render_pipeline,
            attractor_buffer,
            dust_buffer,
            vertex_buffer,
            uniform_buffer,
            dispatch_buffer,
            attractor_bind_group,
            dust_bind_group,
            uniform_bind_group,
            dispatch_bind_group,

            num_particles: dust_particles.len() as u32
        }
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &Uniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    pub fn update(&self, dt: f32, device: &wgpu::Device, queue: &wgpu::Queue, gpu_attractors: &[GpuAttractor]) {
        queue.write_buffer(
            &self.attractor_buffer,
            0,
            bytemuck::cast_slice(&gpu_attractors),
        );

        let max_invocations = WORK_GROUP_SIZE * 65535;
        let mut offset = 0;
        while offset < self.num_particles {
            let remaining = self.num_particles - offset;
            let chunk_size = remaining.min(max_invocations);
            let num_workgroups = (chunk_size + WORK_GROUP_SIZE - 1) / WORK_GROUP_SIZE;

            let params = DispatchParams {
                offset,
                dt,
            };
            queue.write_buffer(&self.dispatch_buffer, 0, bytemuck::bytes_of(&params));

            let mut compute_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

            {
                let mut compute_pass = compute_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                });
                compute_pass.set_pipeline(&self.compute_pipeline);
                compute_pass.set_bind_group(0, &self.dust_bind_group, &[]);
                compute_pass.set_bind_group(1, &self.attractor_bind_group, &[]);
                compute_pass.set_bind_group(2, &self.dispatch_bind_group, &[]);
                compute_pass.dispatch_workgroups(num_workgroups, 1, 1);
            }
            queue.submit(Some(compute_encoder.finish()));
            offset += chunk_size;
        }
    }
}