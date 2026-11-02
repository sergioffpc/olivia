use wgpu::util::{DeviceExt, TextureDataOrder};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent,
    BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrites, Device, Extent3d,
    FilterMode, FragmentState, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderStages, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
    vertex_attr_array,
};

use crate::draw::{Blend, Filter};
use crate::view::Vertex;

const ATTRIBUTES: [VertexAttribute; 3] =
    vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

/// Adds the colour under it to premultiplied colours.
const ADD: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
};

/// How triangles sampling a texture reach the frame.
#[derive(Debug)]
pub(crate) struct Pipeline {
    alpha: RenderPipeline,
    add: RenderPipeline,
    pub(crate) texture_layout: BindGroupLayout,
    linear: BindGroup,
    nearest: BindGroup,
}

impl Pipeline {
    pub(crate) fn new(device: &Device, format: TextureFormat, sample_count: u32) -> Self {
        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("texture"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let sampler_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sampler"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("sprite"),
            bind_group_layouts: &[Some(&texture_layout), Some(&sampler_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(include_wgsl!("sprite.wgsl"));
        let pipeline = |blend: BlendState| {
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("sprite"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[Some(VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &ATTRIBUTES,
                    })],
                },
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState {
                    count: sample_count,
                    ..MultisampleState::default()
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fragment"),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format,
                        blend: Some(blend),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let sampler = |filter: FilterMode| {
            let sampler = device.create_sampler(&SamplerDescriptor {
                label: Some("sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                mag_filter: filter,
                min_filter: filter,
                ..SamplerDescriptor::default()
            });
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("sampler"),
                layout: &sampler_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&sampler),
                }],
            })
        };
        Self {
            alpha: pipeline(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            add: pipeline(ADD),
            linear: sampler(FilterMode::Linear),
            nearest: sampler(FilterMode::Nearest),
            texture_layout,
        }
    }

    /// The pipeline that draws with `blend`.
    pub(crate) fn raw(&self, blend: Blend) -> &RenderPipeline {
        match blend {
            Blend::Alpha => &self.alpha,
            Blend::Add => &self.add,
        }
    }

    /// What binds `filter` to a draw.
    pub(crate) fn sampler(&self, filter: Filter) -> &BindGroup {
        match filter {
            Filter::Linear => &self.linear,
            Filter::Nearest => &self.nearest,
        }
    }

    /// Copies `rgba` to the GPU, premultiplied, and returns what binds it to a draw.
    pub(crate) fn upload(
        &self,
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> BindGroup {
        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("texture"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            TextureDataOrder::default(),
            &premultiplied(rgba),
        );
        let view = texture.create_view(&TextureViewDescriptor::default());
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("texture"),
            layout: &self.texture_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view),
            }],
        })
    }
}

pub(crate) fn premultiplied(rgba: &[u8]) -> Vec<u8> {
    let mut premultiplied = rgba.to_vec();
    for pixel in premultiplied.as_chunks_mut::<4>().0 {
        let [r, g, b, a] = pixel.map(u16::from);
        let scaled = |channel: u16| u8::try_from((channel * a + 127) / 255).unwrap_or(u8::MAX);
        *pixel = [scaled(r), scaled(g), scaled(b), pixel[3]];
    }
    premultiplied
}
