use eframe::egui::{self, TextureHandle};
use rs_gb_core::DebugInfo;

const DEBUG_WIDTH: usize = 16 * 8 + 16 + 1; 
const DEBUG_HEIGHT: usize = 24 * 8 + 24 + 1;

const COLORS: [u32; 4] = [0xFFFFFFFF, 0xFFAAAAAA, 0xFF555555, 0xFF000000];

pub struct Debugger {
    vram_debug: bool,
    tilemap: [u8; DEBUG_WIDTH * DEBUG_HEIGHT * 4],
    tile_texture: TextureHandle,
}

impl Debugger {
    pub fn new(cc: &eframe::CreationContext) -> Debugger {
        let tilemap = [0; DEBUG_WIDTH * DEBUG_HEIGHT * 4];

        let tile_texture = cc.egui_ctx.load_texture(
            "tile_map",
            egui::ColorImage::from_rgba_unmultiplied([DEBUG_WIDTH, DEBUG_HEIGHT], &tilemap),
            egui::TextureOptions::NEAREST
        );

        Debugger { 
            vram_debug: false,
            tilemap,
            tile_texture,
        }
    }

    /// Renders the entirety of the debugger window
    pub fn render(&mut self, ctx: &egui::Context, debug_info: DebugInfo) -> bool {
        let mut stay_open = true;

        if self.vram_debug && debug_info.vram_updated() { self.draw_vram(&debug_info); }

        egui::SidePanel::right("tiles")
            .exact_width(450.0)
            .show(ctx, |ui| {
            ui.heading("VRAM Tiles Visualizer");

            if ui.checkbox(&mut self.vram_debug, "Enable VRAM vizualization").changed() {
                if self.vram_debug { self.draw_vram(&debug_info); }
            }

            if self.vram_debug {
                ui.vertical_centered(|ui| {
                    let scale = (ui.available_height() / DEBUG_HEIGHT as f32).floor();
                    let image_widget = egui::Image::new(&self.tile_texture)
                        .fit_to_original_size(scale);

                    ui.add(image_widget);
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Cartridge Type");
            ui.label(debug_info.game_cartridge_type());

            if ui.input(|i| i.viewport().close_requested()) {
                // Tell parent to close us.
                stay_open = false;
            }
        });
        stay_open
    }

    fn draw_vram(&mut self, debug_info: &DebugInfo) {
        let tiles = debug_info.get_tiles();

        assert!(tiles.len() == 512);
        
        for y in 0..24 {
            for x in 0..16 {
                let tile = &tiles[x + y * 16];
                display_tile(&mut self.tilemap, x * 9 + 1, y * 9 + 1, tile);
            }
        }

        self.tile_texture.set(
            egui::ColorImage::from_rgba_unmultiplied([DEBUG_WIDTH, DEBUG_HEIGHT], &self.tilemap),
            egui::TextureOptions::NEAREST
        );
    }
}

fn display_tile(buffer: &mut [u8], start_x: usize, start_y: usize, tile: &[u8; 16]) {
    for tile_y in (0..16).step_by(2) {
        let b1 = tile[tile_y];
        let b2 = tile[tile_y + 1];

        for bit in (0..8).rev() {
            let low = ((b1 & (1 << bit)) != 0) as usize;
            let high = ((b2 & (1 << bit)) != 0) as usize;

            let color = high << 1 | low;
            let color_u32 = COLORS[color];

            let a = ((color_u32 >> 24) & 0xFF) as u8;
            let r = ((color_u32 >> 16) & 0xFF) as u8;
            let g = ((color_u32 >> 8) & 0xFF) as u8;
            let b = (color_u32 & 0xFF) as u8;

            let x = start_x + (7 - bit);
            let y = start_y + tile_y / 2;

            let base_idx = (x + DEBUG_WIDTH * y) * 4;
            
            buffer[base_idx]     = r;
            buffer[base_idx + 1] = g;
            buffer[base_idx + 2] = b;
            buffer[base_idx + 3] = a;
        }
    }
}