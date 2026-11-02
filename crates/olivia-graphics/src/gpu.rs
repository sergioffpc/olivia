use std::sync::Arc;

use olivia_math::Color;
use olivia_window::RawWindow;
use tracing::{debug, info, trace, trace_span, warn};
use wgpu::{
    Color as ClearColor, CommandEncoderDescriptor, CurrentSurfaceTexture, Device, DeviceDescriptor,
    Extent3d, Features, Instance, InstanceDescriptor, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

use crate::Error;
use crate::graphics::Config;

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
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    sample_count: u32,
    /// What a frame renders to before resolving to the surface; none without MSAA.
    multisampled: Option<TextureView>,
    /// Read for its size each frame, so a resize needs no event.
    window: Arc<RawWindow>,
}

impl Gpu {
    pub(crate) fn new(window: &Arc<RawWindow>, config: Config) -> Result<Self, Error> {
        let display = Box::new(Arc::clone(window));
        let instance = Instance::new(InstanceDescriptor::new_with_display_handle_from_env(
            display,
        ));
        let surface = instance
            .create_surface(Arc::clone(window))
            .map_err(Error::no_adapter)?;
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..RequestAdapterOptions::default()
        }))
        .map_err(Error::no_adapter)?;
        info!(adapter = adapter.get_info().name, "adapter chosen");

        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            // Counts above 4 are specific to the adapter and need this.
            required_features: adapter.features()
                & Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            required_limits: adapter.limits(),
            ..DeviceDescriptor::default()
        }))
        .map_err(|error| Error::NoDevice(error.into()))?;

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

        let mut gpu = Self {
            device,
            queue,
            surface,
            config: surface_config,
            sample_count,
            multisampled: None,
            window: Arc::clone(window),
        };
        gpu.configure();
        Ok(gpu)
    }

    /// Clears the window to `color` and shows it, returning whether it was shown.
    pub(crate) fn present(&mut self, color: Color) -> bool {
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
            // Whatever stopped this frame may be gone by the next one.
            _ => {
                trace!("frame skipped");
                return false;
            }
        };

        let _span = trace_span!("submit").entered();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let (attachment, resolve_target) = match &self.multisampled {
            Some(multisampled) => (multisampled, Some(&view)),
            None => (&view, None),
        };
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        encoder.begin_render_pass(&RenderPassDescriptor {
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
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(frame);
        true
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
