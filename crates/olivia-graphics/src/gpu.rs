use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bytemuck::cast_slice;
use olivia_math::{Color, Vec2};
use olivia_window::RawWindow;
use tracing::{debug, info, trace, trace_span, warn};
use wgpu::util::{BufferInitDescriptor, DeviceExt, new_instance_with_webgpu_detection};
use wgpu::{
    BindGroup, BufferUsages, Color as ClearColor, CommandEncoderDescriptor, CurrentSurfaceTexture,
    Device, DeviceDescriptor, Extent3d, Features, Instance, InstanceDescriptor, Limits, LoadOp,
    Operations, PollType, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

use crate::Error;
use crate::atlas::AtlasGpu;
use crate::graphics::Config;
use crate::pipeline::Pipeline;
use crate::view::Mesh;

/// The largest of `supported` that does not exceed `requested`, or 1, which disables MSAA.
#[doc(hidden)]
#[must_use]
pub fn sample_count(requested: u32, supported: &[u32]) -> u32 {
    let below = supported
        .iter()
        .copied()
        .filter(|&count| count <= requested);
    below.max().unwrap_or(1)
}

/// The GPU, drawing to one window.
#[derive(Debug)]
pub(crate) struct Gpu {
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    /// Set by `wgpu` when the device is lost, from whichever thread finds out.
    device_lost: Arc<AtomicBool>,
    instance: Instance,
    surface: Surface<'static>,
    /// Whether the last frame found the surface lost.
    surface_lost: bool,
    config: SurfaceConfiguration,
    sample_count: u32,
    pipeline: Pipeline,
    /// What a frame renders to before resolving to the surface; none without MSAA.
    multisampled: Option<TextureView>,
    /// Read for its size each frame, so a resize needs no event.
    pub(crate) window: Arc<RawWindow>,
}

impl Gpu {
    pub(crate) async fn new(window: &Arc<RawWindow>, config: Config) -> Result<Self, Error> {
        let display = Box::new(Arc::clone(window));
        // A browser may have WebGPU but no adapter for it, as Chrome on Linux does, so WebGL2 is
        // the fallback there too.
        let instance = new_instance_with_webgpu_detection(
            InstanceDescriptor::new_with_display_handle_from_env(display),
        )
        .await;
        let surface = instance
            .create_surface(Arc::clone(window))
            .map_err(Error::no_adapter)?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..RequestAdapterOptions::default()
            })
            .await
            .map_err(Error::no_adapter)?;
        info!(adapter = adapter.get_info().name, "adapter chosen");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                // Counts above 4 are specific to the adapter and need this.
                required_features: adapter.features()
                    & Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                // WebGL2's minimums, so a game behaves the same on every GPU. Only the texture
                // sizes come from the adapter, for the surface and its targets; game textures are
                // capped by `Graphics` itself.
                required_limits: Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                ..DeviceDescriptor::default()
            })
            .await
            .map_err(|error| Error::NoDevice(error.into()))?;
        let device_lost = Arc::new(AtomicBool::new(false));
        let lost = Arc::clone(&device_lost);
        device.set_device_lost_callback(move |_, _| lost.store(true, Ordering::Relaxed));

        let size = window.inner_size();
        let mut surface_config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or(Error::NoAdapter(None))?;
        // Channels reach the screen as the game wrote them, with no conversion to linear.
        let formats = surface.get_capabilities(&adapter).formats;
        surface_config.format = formats
            .into_iter()
            .find(|format| {
                matches!(
                    format,
                    TextureFormat::Bgra8Unorm | TextureFormat::Rgba8Unorm
                )
            })
            .unwrap_or(surface_config.format);
        surface_config.present_mode = if config.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };

        let supported = adapter
            .get_texture_format_features(surface_config.format)
            .flags
            .supported_sample_counts();
        let sample_count = sample_count(config.msaa, &supported);
        if sample_count != config.msaa {
            warn!(
                requested = config.msaa,
                sample_count, "MSAA sample count not supported"
            );
        }

        let pipeline = Pipeline::new(&device, surface_config.format, sample_count);
        let mut gpu = Self {
            device,
            queue,
            device_lost,
            instance,
            surface,
            surface_lost: false,
            config: surface_config,
            sample_count,
            pipeline,
            multisampled: None,
            window: Arc::clone(window),
        };
        gpu.configure();
        Ok(gpu)
    }

    /// Whether the device is lost, and everything made from it with it.
    pub(crate) fn is_device_lost(&self) -> bool {
        // A destroyed device only reports its loss from a poll. A failed poll has nothing to add.
        let _ = self.device.poll(PollType::Poll);
        self.device_lost.load(Ordering::Relaxed)
    }

    /// Replaces the surface if the last frame found it lost.
    pub(crate) fn recreate_lost_surface(&mut self) -> Result<(), Error> {
        if !mem::take(&mut self.surface_lost) {
            return Ok(());
        }
        // Replacing it drops the old swapchain, and `configure` then builds the new one.
        self.surface = self
            .instance
            .create_surface(Arc::clone(&self.window))
            .map_err(Error::no_adapter)?;
        debug!("surface recreated");
        self.configure();
        Ok(())
    }

    /// The window's logical size.
    pub(crate) fn viewport(&self) -> Vec2 {
        let size = self.window.inner_size();
        let scale_factor = self.window.scale_factor();
        let size = size.to_logical::<f32>(scale_factor);
        Vec2::new(size.width, size.height)
    }

    /// Clears the window to `color`, draws `mesh`, whose vertices are in a window of logical size
    /// `viewport`, and shows it, returning whether it was shown.
    pub(crate) fn present(&mut self, color: Color, mesh: &Mesh, viewport: Vec2) -> bool {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return false;
        }
        if (size.width, size.height) != (self.config.width, self.config.height) {
            self.config.width = size.width;
            self.config.height = size.height;
            self.configure();
        }

        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) | CurrentSurfaceTexture::Suboptimal(frame) => {
                frame
            }
            CurrentSurfaceTexture::Outdated => {
                self.configure();
                return false;
            }
            CurrentSurfaceTexture::Lost => {
                self.surface_lost = true;
                return false;
            }
            // Whatever stopped this frame may be gone by the next one.
            _ => {
                trace!("frame skipped");
                return false;
            }
        };

        let _span = trace_span!("submit", batches = mesh.batches.len()).entered();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let (attachment, resolve_target) = match &self.multisampled {
            Some(multisampled) => (multisampled, Some(&view)),
            None => (&view, None),
        };
        // A buffer cannot be empty, and a frame with no view only clears.
        let vertices = (!mesh.vertices.is_empty()).then(|| {
            self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("frame vertices"),
                contents: cast_slice(&mesh.clip_vertices(viewport)),
                usage: BufferUsages::VERTEX,
            })
        });
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: attachment,
                depth_slice: None,
                resolve_target,
                ops: Operations {
                    load: LoadOp::Clear(clear_color(color)),
                    // A resolved image needs no copy of its samples.
                    store: if resolve_target.is_some() {
                        StoreOp::Discard
                    } else {
                        StoreOp::Store
                    },
                },
            })],
            ..RenderPassDescriptor::default()
        });
        if let Some(vertices) = &vertices {
            pass.set_vertex_buffer(0, vertices.slice(..));
            for batch in &mesh.batches {
                pass.set_pipeline(self.pipeline.raw(batch.blend));
                pass.set_bind_group(0, batch.image.bind_group().as_ref(), &[]);
                pass.set_bind_group(1, self.pipeline.sampler(batch.filter), &[]);
                pass.draw(batch.vertices.clone(), 0..1);
            }
        }
        drop(pass);
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(frame);
        true
    }

    /// Copies `rgba` to the GPU and returns what binds it to a draw.
    pub(crate) fn upload(&self, width: u32, height: u32, rgba: &[u8]) -> BindGroup {
        self.pipeline
            .upload(&self.device, &self.queue, width, height, rgba)
    }

    /// What a font needs to keep its glyphs on the GPU.
    pub(crate) fn atlas_gpu(&self) -> AtlasGpu {
        AtlasGpu {
            device: self.device.clone(),
            queue: self.queue.clone(),
            texture_layout: self.pipeline.texture_layout.clone(),
        }
    }

    /// A multisampled texture the size of the surface.
    fn multisampled_view(&self) -> TextureView {
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some("multisampled frame"),
            size: Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: self.sample_count,
            dimension: TextureDimension::D2,
            format: self.config.format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&TextureViewDescriptor::default())
    }

    /// Applies `config` to the surface and builds what depends on its size.
    fn configure(&mut self) {
        self.surface.configure(&self.device, &self.config);
        self.multisampled = (self.sample_count > 1).then(|| self.multisampled_view());
        debug!(
            width = self.config.width,
            height = self.config.height,
            "surface reconfigured"
        );
    }
}

fn clear_color(color: Color) -> ClearColor {
    ClearColor {
        r: f64::from(color.r),
        g: f64::from(color.g),
        b: f64::from(color.b),
        a: f64::from(color.a),
    }
}
