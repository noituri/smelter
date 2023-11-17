use compositor_common::scene::Resolution;

use crate::wgpu::WgpuCtx;

use super::base::Texture;

#[derive(Debug)]
pub struct RGBATexture(Texture);

impl RGBATexture {
    pub fn new(ctx: &WgpuCtx, resolution: Resolution) -> Self {
        Self(Texture::new(
            ctx,
            None,
            wgpu::Extent3d {
                width: resolution.width as u32,
                height: resolution.height as u32,
                depth_or_array_layers: 1,
            },
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        ))
    }

    pub fn new_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("single texture bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                ty: Texture::DEFAULT_BINDING_TYPE,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
            }],
        })
    }

    pub(super) fn new_bind_group(
        &self,
        ctx: &WgpuCtx,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.0.view),
            }],
        })
    }

    pub fn upload(&self, ctx: &WgpuCtx, data: &[u8]) {
        self.0.upload_data(&ctx.queue, data, 4);
    }

    pub fn new_download_buffer(&self, ctx: &WgpuCtx) -> wgpu::Buffer {
        self.0.new_download_buffer(ctx)
    }

    pub fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, buffer: &wgpu::Buffer) {
        self.0.copy_to_buffer(encoder, buffer);
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.0.size()
    }

    pub fn texture(&self) -> &Texture {
        &self.0
    }
}
