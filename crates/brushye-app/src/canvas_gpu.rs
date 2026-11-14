//! GPU side of the canvas display (D-012 4): composited tiles become textures and are drawn
//! with the view transform. The GPU only displays; it never composites the document.
//!
//! - Textures are `Rgba8Unorm` (not sRGB) holding the CPU pixels as they are.
//! - Sampling is nearest at zoom ≥ 100 % and linear below.
//! - Opaque pixels are written unchanged. Transparent areas show a checkerboard, mixed only for
//!   display.
//!
//! So at 100 %, unrotated, with the origin on a whole pixel, every screen pixel equals its CPU
//! pixel. The offscreen test in `tests/display.rs` checks this.

use std::collections::HashMap;

use eframe::egui_wgpu::wgpu;
use wgpu::util::DeviceExt as _;

use crate::display::{TileKey, Upload};

const SHADER: &str = r#"
struct View {
    m: vec4<f32>,  // row-major 2x2: canvas -> screen
    t: vec4<f32>,  // translation (x, y), target size (w, h), physical px
    c: vec4<f32>,  // checker cell px, light, dark, unused
};
@group(0) @binding(0) var<uniform> view: View;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var smp: sampler;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32, @location(0) rect: vec4<f32>) -> VOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
    let c = corners[vi];
    let p = rect.xy + c * rect.zw;
    let s = vec2<f32>(view.m.x * p.x + view.m.y * p.y, view.m.z * p.x + view.m.w * p.y) + view.t.xy;
    var o: VOut;
    o.pos = vec4<f32>(s.x / view.t.z * 2.0 - 1.0, 1.0 - s.y / view.t.w * 2.0, 0.0, 1.0);
    o.uv = c;
    return o;
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let col = textureSample(tex, smp, in.uv);
    if (col.a >= 1.0) {
        return vec4<f32>(col.rgb, 1.0);
    }
    let q = floor(in.pos.xy / view.c.x);
    let even = ((i32(q.x) + i32(q.y)) & 1) == 0;
    let k = select(view.c.z, view.c.y, even);
    return vec4<f32>(mix(vec3<f32>(k, k, k), col.rgb, col.a), 1.0);
}
"#;

/// View parameters for one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewUniform {
    /// Row-major 2×2 canvas → screen matrix (physical px).
    pub m: [f32; 4],
    /// Screen position of the canvas origin (physical px, in the render target).
    pub origin: [f32; 2],
    /// Render target size in physical px.
    pub target: [f32; 2],
    /// Nearest (true) or linear sampling.
    pub nearest: bool,
}

struct GpuTile {
    w: u32,
    h: u32,
    texture: wgpu::Texture,
    near: wgpu::BindGroup,
    linear: wgpu::BindGroup,
}

/// All GPU resources of the canvas display.
pub struct CanvasGpu {
    pipeline: wgpu::RenderPipeline,
    tile_layout: wgpu::BindGroupLayout,
    view_buf: wgpu::Buffer,
    view_bg: wgpu::BindGroup,
    near: wgpu::Sampler,
    linear: wgpu::Sampler,
    instances: wgpu::Buffer,
    instance_cap: u64,
    tiles: HashMap<TileKey, GpuTile>,
    /// Tiles to draw this frame, in instance order.
    order: Vec<TileKey>,
    nearest: bool,
}

const INSTANCE_BYTES: u64 = 16;

impl CanvasGpu {
    pub fn new(device: &wgpu::Device, target: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("brushye canvas"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let view_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("canvas view"),
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
        let tile_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("canvas tile"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("canvas"),
            bind_group_layouts: &[Some(&view_layout), Some(&tile_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("canvas"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: INSTANCE_BYTES,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    }],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let view_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("canvas view"),
            size: 48,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let view_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("canvas view"),
            layout: &view_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_buf.as_entire_binding(),
            }],
        });
        let sampler = |filter| {
            device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("canvas"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: filter,
                min_filter: filter,
                ..Default::default()
            })
        };
        let instance_cap = 256;
        Self {
            pipeline,
            tile_layout,
            view_buf,
            view_bg,
            near: sampler(wgpu::FilterMode::Nearest),
            linear: sampler(wgpu::FilterMode::Linear),
            instances: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("canvas tiles"),
                size: instance_cap * INSTANCE_BYTES,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            instance_cap,
            tiles: HashMap::new(),
            order: Vec::new(),
            nearest: true,
        }
    }

    /// Upload changed tiles (after dropping everything when `reset`).
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uploads: &[Upload],
        reset: bool,
    ) {
        if reset {
            self.tiles.clear();
        }
        for u in uploads {
            let fits = self
                .tiles
                .get(&u.key)
                .is_some_and(|t| t.w == u.w && t.h == u.h);
            if !fits {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("canvas tile"),
                    size: wgpu::Extent3d {
                        width: u.w,
                        height: u.h,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let tv = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let bg = |s: &wgpu::Sampler| {
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("canvas tile"),
                        layout: &self.tile_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&tv),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(s),
                            },
                        ],
                    })
                };
                let tile = GpuTile {
                    w: u.w,
                    h: u.h,
                    near: bg(&self.near),
                    linear: bg(&self.linear),
                    texture,
                };
                self.tiles.insert(u.key, tile);
            }
            if let Some(t) = self.tiles.get(&u.key) {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &t.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &u.px,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * u.w),
                        rows_per_image: Some(u.h),
                    },
                    wgpu::Extent3d {
                        width: u.w,
                        height: u.h,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }

    /// Set the view and the tile list for the next [`CanvasGpu::draw`].
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, v: &ViewUniform) {
        let floats: [f32; 12] = [
            v.m[0],
            v.m[1],
            v.m[2],
            v.m[3],
            v.origin[0],
            v.origin[1],
            v.target[0],
            v.target[1],
            8.0,
            0.86,
            0.72,
            0.0,
        ];
        let bytes: Vec<u8> = floats.iter().flat_map(|f| f.to_le_bytes()).collect();
        queue.write_buffer(&self.view_buf, 0, &bytes);
        self.nearest = v.nearest;
        let mut keys: Vec<TileKey> = self.tiles.keys().copied().collect();
        keys.sort_unstable();
        let mut inst = Vec::with_capacity(keys.len() * INSTANCE_BYTES as usize);
        for k in &keys {
            let t = &self.tiles[k];
            for f in [
                k.0 as f32 * 256.0,
                k.1 as f32 * 256.0,
                t.w as f32,
                t.h as f32,
            ] {
                inst.extend_from_slice(&f.to_le_bytes());
            }
        }
        let need = keys.len() as u64;
        if need > self.instance_cap {
            self.instance_cap = need.next_power_of_two();
            self.instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("canvas tiles"),
                contents: &{
                    let mut v = inst.clone();
                    v.resize((self.instance_cap * INSTANCE_BYTES) as usize, 0);
                    v
                },
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        } else if !inst.is_empty() {
            queue.write_buffer(&self.instances, 0, &inst);
        }
        self.order = keys;
    }

    /// Draw the canvas into the current render pass.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.view_bg, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        for (i, k) in self.order.iter().enumerate() {
            if let Some(t) = self.tiles.get(k) {
                pass.set_bind_group(1, if self.nearest { &t.near } else { &t.linear }, &[]);
                let i = i as u32;
                pass.draw(0..6, i..i + 1);
            }
        }
    }
}

/// Tiles waiting for the GPU. Shared between the app and the paint callback, so tiles are
/// never lost when a frame isn't painted (e.g. before the window is shown).
#[derive(Default)]
pub struct Pending {
    uploads: HashMap<TileKey, Upload>,
    reset: bool,
}

impl Pending {
    /// Queue uploads; `reset` drops everything already on the GPU first.
    pub fn push(&mut self, uploads: Vec<Upload>, reset: bool) {
        if reset {
            self.uploads.clear();
            self.reset = true;
        }
        for u in uploads {
            self.uploads.insert(u.key, u);
        }
    }

    fn take(&mut self) -> (Vec<Upload>, bool) {
        let mut v: Vec<Upload> = self.uploads.drain().map(|(_, u)| u).collect();
        v.sort_by_key(|u| u.key);
        (v, std::mem::take(&mut self.reset))
    }
}

/// The egui paint callback that draws the canvas inside the UI's render pass.
pub struct CanvasCallback {
    pub pending: std::sync::Arc<std::sync::Mutex<Pending>>,
    pub view: ViewUniform,
}

impl eframe::egui_wgpu::CallbackTrait for CanvasCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(gpu) = resources.get_mut::<CanvasGpu>() {
            let (uploads, reset) = match self.pending.lock() {
                Ok(mut p) => p.take(),
                Err(_) => (Vec::new(), false),
            };
            gpu.upload(device, queue, &uploads, reset);
            let mut v = self.view;
            v.target = [
                screen.size_in_pixels[0] as f32,
                screen.size_in_pixels[1] as f32,
            ];
            gpu.prepare(device, queue, &v);
        }
        Vec::new()
    }

    fn paint(
        &self,
        info: eframe::egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        // Draw in whole-window coordinates; egui's scissor still clips to the canvas area.
        let [w, h] = info.screen_size_px;
        pass.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
        if let Some(gpu) = resources.get::<CanvasGpu>() {
            gpu.draw(pass);
        }
    }
}
