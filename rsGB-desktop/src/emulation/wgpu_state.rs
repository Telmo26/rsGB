use eframe::wgpu;

use crate::settings::{FRAME_SIZE, XRES, YRES};

pub struct WgpuState {
    texture: wgpu::Texture,
    texture_id: eframe::egui::TextureId
}

impl WgpuState {
    pub fn new(cc: &eframe::CreationContext) -> WgpuState {
        let wgpu_state = cc.wgpu_render_state
            .as_ref()
            .expect("eframe must be run with Renderer::Wgpu");

        let device = &wgpu_state.device;
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gb_framebuffer"),
            size: wgpu::Extent3d { width: XRES as u32, height: YRES as u32, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[]
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_id = wgpu_state.renderer.write().register_native_texture(
            device, 
            &view, 
            wgpu::FilterMode::Nearest
        );

        WgpuState { texture, texture_id }
    }

    pub fn update(&mut self, frame: &eframe::Frame, pixels: &[u32; FRAME_SIZE]) {
        let render_state = frame.wgpu_render_state()
            .expect("eframe must be run with Renderer::Wgpu");

        let bytes = bytemuck::cast_slice(pixels);

        render_state.queue.write_texture(
            self.texture.as_image_copy(), 
            bytes, 
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * XRES as u32),
                rows_per_image: Some(YRES as u32)
            }, 
            wgpu::Extent3d { width: XRES as u32, height: YRES as u32, depth_or_array_layers: 1 }
        );
    }

    pub fn texture_id(&self) -> &eframe::egui::TextureId {
        &self.texture_id
    }
}