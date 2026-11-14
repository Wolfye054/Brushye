//! Display test (D-012 4): at 100 %, unrotated, the pixels the GPU shows equal the CPU
//! renderer's pixels exactly. Renders offscreen on a software adapter when one is available
//! (lavapipe on Linux CI, WARP on Windows). Set `BRUSHYE_REQUIRE_GPU=1` to make a missing
//! adapter a failure instead of a skip (CI does).

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use brushye_app::canvas_gpu::{CanvasGpu, ViewUniform};
use brushye_app::display::Display;
use brushye_command::Session;
use eframe::egui_wgpu::wgpu;

/// Minimal executor: wgpu's native futures complete when polled after the device is polled.
fn block_on<F: Future>(f: F, device: Option<&wgpu::Device>) -> F::Output {
    let mut cx = Context::from_waker(Waker::noop());
    let mut f = pin!(f);
    loop {
        if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        if let Some(d) = device {
            let _ = d.poll(wgpu::PollType::wait_indefinitely());
        }
        std::thread::yield_now();
    }
}

fn device() -> Option<(wgpu::Device, wgpu::Queue, String)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let try_adapter = |fallback: bool| {
        block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: fallback,
                compatible_surface: None,
                ..Default::default()
            }),
            None,
        )
        .ok()
    };
    // Prefer a software adapter (deterministic, available in CI); else any adapter.
    let adapter = try_adapter(true).or_else(|| try_adapter(false))?;
    let name = format!("{:?}", adapter.get_info());
    let (d, q) = block_on(
        adapter.request_device(&wgpu::DeviceDescriptor::default()),
        None,
    )
    .ok()?;
    Some((d, q, name))
}

fn render(
    gpu: &mut CanvasGpu,
    d: &wgpu::Device,
    q: &wgpu::Queue,
    w: u32,
    h: u32,
    view: ViewUniform,
) -> Vec<u8> {
    let target = d.create_texture(&wgpu::TextureDescriptor {
        label: Some("target"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let tv = target.create_view(&wgpu::TextureViewDescriptor::default());
    gpu.prepare(d, q, &view);
    let mut enc = d.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("canvas"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &tv,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        gpu.draw(&mut pass);
    }
    let row = (4 * w).div_ceil(256) * 256;
    let buf = d.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(row * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    q.submit([enc.finish()]);
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |r| r.unwrap_or_else(|e| panic!("{e}")));
    let _ = d.poll(wgpu::PollType::wait_indefinitely());
    let data = slice.get_mapped_range().unwrap_or_else(|e| panic!("{e:?}"));
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let s = (y * row) as usize;
        out.extend_from_slice(&data[s..s + (4 * w) as usize]);
    }
    out
}

#[test]
fn gpu_shows_exactly_the_cpu_pixels_at_100_percent() {
    let Some((d, q, name)) = device() else {
        assert!(
            std::env::var_os("BRUSHYE_REQUIRE_GPU").is_none(),
            "no wgpu adapter available"
        );
        eprintln!("skipped: no wgpu adapter");
        return;
    };
    eprintln!("adapter: {name}");
    // A document spanning several tiles with clipped edge tiles, blend modes and strokes.
    let mut s = Session::new();
    s.run_script(
        "doc.new 530 300 paper=#f4ecd8
layer.new L1
layer.fill L1 color=#2a6f9733
layer.new L2
brush.stroke L2 color=#c0392b size=40 hardness=2 samples=[20,20,1,0;500,280,0.5,9000]
brush.stroke L2 color=#1d3557 size=7 aa=strong samples=[10,290,1,0;520,15,1,9000;265,150,1,18000]
layer.set L2 blend=multiply opacity=85",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let doc = s.document().unwrap_or_else(|| panic!("no document"));
    let mut disp = Display::default();
    let (uploads, reset) = disp.update(&doc);
    let mut gpu = CanvasGpu::new(&d, wgpu::TextureFormat::Rgba8Unorm);
    gpu.upload(&d, &q, &uploads, reset);
    let (w, h) = (doc.width, doc.height);
    let shown = render(
        &mut gpu,
        &d,
        &q,
        w,
        h,
        ViewUniform {
            m: [1.0, 0.0, 0.0, 1.0],
            origin: [0.0, 0.0],
            target: [w as f32, h as f32],
            nearest: true,
        },
    );
    let cpu = brushye_command::core::render::flatten(&doc);
    let wrong = shown
        .as_chunks::<4>()
        .0
        .iter()
        .zip(cpu.px.as_chunks::<4>().0)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(wrong, 0, "{wrong} of {} pixels differ", w * h);

    // Shifted by a whole pixel offset into a bigger target: still exact, and the area outside
    // the canvas is untouched.
    let shown = render(
        &mut gpu,
        &d,
        &q,
        w + 20,
        h + 10,
        ViewUniform {
            m: [1.0, 0.0, 0.0, 1.0],
            origin: [13.0, 7.0],
            target: [(w + 20) as f32, (h + 10) as f32],
            nearest: true,
        },
    );
    for y in 0..h {
        for x in 0..w {
            let i = (((y + 7) * (w + 20) + x + 13) * 4) as usize;
            assert_eq!(&shown[i..i + 4], &cpu.pixel(x, y), "({x}, {y})");
        }
    }
    assert_eq!(&shown[0..4], &[0, 0, 0, 255]);
}
