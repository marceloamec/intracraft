//! The card, the BIMCraft way (`crates/ui-egui/src/gpu.rs` there): the viewer's own pipeline draws into its own
//! colour and depth targets, and egui shows the picture as a texture. What BIMCraft's view lacks for a simulation, and
//! this adds, is instancing: one mesh drawn many times, with the instances' poses written every frame.

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use eframe::egui_wgpu::{self, RenderState, wgpu};
use egui::mutex::RwLock;
use glam::Mat4;

use crate::mesh::{MeshData, Vertex};

const COLOR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// One drawn copy of a mesh: where it is (turn and place), its scale along its own axes, and its tint.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Instance {
    pub model: [[f32; 4]; 4],
    pub scale: [f32; 4],
    pub tint: [f32; 4],
}

impl Instance {
    pub fn new(model: Mat4, scale: [f32; 3], tint: [f32; 3]) -> Instance {
        Instance {
            model: model.to_cols_array_2d(),
            scale: [scale[0], scale[1], scale[2], 0.0],
            tint: [tint[0], tint[1], tint[2], 1.0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    sun: [f32; 4],
}

/// A mesh on the card and its instances.
struct Batch {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
    instances: wgpu::Buffer,
    capacity: usize,
    count: u32,
}

struct Targets {
    size: (u32, u32),
    color: wgpu::TextureView,
    depth: wgpu::TextureView,
}

pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Arc<RwLock<egui_wgpu::Renderer>>,
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    bind: wgpu::BindGroup,
    batches: Vec<Batch>,
    targets: Option<Targets>,
    texture: Option<egui::TextureId>,
    pub adapter: String,
    pub backend: String,
}

impl Gpu {
    pub fn new(state: &RenderState) -> Gpu {
        let device = state.device.clone();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("r5b"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let group = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("r5b-globals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("r5b"),
            bind_group_layouts: &[Some(&group)],
            immediate_size: 0,
        });
        let vertex_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];
        let instance_attributes = wgpu::vertex_attr_array![
            3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4
        ];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("r5b"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    Some(wgpu::VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attributes,
                    }),
                    Some(wgpu::VertexBufferLayout {
                        array_stride: size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &instance_attributes,
                    }),
                ],
            },
            primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Back), ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: COLOR,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("r5b-globals"),
            size: size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("r5b-globals"),
            layout: &group,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() }],
        });
        let info = state.adapter.get_info();
        Gpu {
            device,
            queue: state.queue.clone(),
            renderer: state.renderer.clone(),
            pipeline,
            globals,
            bind,
            batches: Vec::new(),
            targets: None,
            texture: None,
            adapter: info.name,
            backend: format!("{:?}", info.backend),
        }
    }

    /// Puts a mesh on the card with room for `capacity` instances; returns its batch number.
    pub fn add_batch(&mut self, mesh: &MeshData, capacity: usize) -> usize {
        let buffer = |label, bytes: &[u8], usage| {
            let b = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: (bytes.len() as u64).max(16).next_multiple_of(4),
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&b, 0, bytes);
            b
        };
        let vertices = buffer("r5b-vertices", bytemuck::cast_slice(&mesh.vertices), wgpu::BufferUsages::VERTEX);
        let indices = buffer("r5b-indices", bytemuck::cast_slice(&mesh.indices), wgpu::BufferUsages::INDEX);
        let instances = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("r5b-instances"),
            size: (capacity.max(1) * size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.batches.push(Batch {
            vertices,
            indices,
            index_count: mesh.indices.len() as u32,
            instances,
            capacity,
            count: 0,
        });
        self.batches.len() - 1
    }

    /// Writes a batch's instances (at most its capacity).
    pub fn set_instances(&mut self, batch: usize, instances: &[Instance]) {
        if let Some(b) = self.batches.get_mut(batch) {
            let n = instances.len().min(b.capacity);
            self.queue.write_buffer(&b.instances, 0, bytemuck::cast_slice(&instances[..n]));
            b.count = n as u32;
        }
    }

    /// Draws every batch at `size` pixels seen through `view_proj`, and returns the picture as an egui texture.
    pub fn draw(&mut self, size: (u32, u32), view_proj: Mat4, sun: [f32; 3]) -> Option<egui::TextureId> {
        let size = (size.0.max(1), size.1.max(1));
        if self.targets.as_ref().is_none_or(|t| t.size != size) {
            self.targets = Some(self.make_targets(size));
            if let (Some(id), Some(t)) = (self.texture, &self.targets) {
                self.renderer.write().update_egui_texture_from_wgpu_texture(
                    &self.device,
                    &t.color,
                    wgpu::FilterMode::Linear,
                    id,
                );
            }
        }
        let targets = self.targets.as_ref()?;
        let globals = Globals { view_proj: view_proj.to_cols_array_2d(), sun: [sun[0], sun[1], sun[2], 0.0] };
        self.queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("r5b") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("r5b"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.color,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.62, g: 0.7, b: 0.78, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &targets.depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind, &[]);
            for b in self.batches.iter().filter(|b| b.count > 0) {
                pass.set_vertex_buffer(0, b.vertices.slice(..));
                pass.set_vertex_buffer(1, b.instances.slice(..));
                pass.set_index_buffer(b.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..b.index_count, 0, 0..b.count);
            }
        }
        self.queue.submit([encoder.finish()]);
        if self.texture.is_none() {
            self.texture = Some(self.renderer.write().register_native_texture(
                &self.device,
                &targets.color,
                wgpu::FilterMode::Linear,
            ));
        }
        self.texture
    }

    fn make_targets(&self, size: (u32, u32)) -> Targets {
        let texture = |label, format, usage| {
            self.device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };
        Targets {
            size,
            color: texture(
                "r5b-color",
                COLOR,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            depth: texture("r5b-depth", DEPTH, wgpu::TextureUsages::RENDER_ATTACHMENT),
        }
    }
}
