use eframe::egui::{self, RichText, TextureHandle};
use rsgb_core::{DebugInfo, GameInfo};

/// I want a 16 by 24 grid for displaying tiles, that are 8 pixels square.
/// There is also a one pixel margin between the tiles
const DEBUG_WIDTH: usize = 16 * (8 + 1) + 1; 
const DEBUG_HEIGHT: usize = 24 * (8 + 1) + 1;

const COLORS: [u32; 4] = [0xFFFFFFFF, 0xFFAAAAAA, 0xFF555555, 0xFF000000];

pub struct Debugger {
    vram_debug: bool,
    tilemap: [u8; DEBUG_WIDTH * DEBUG_HEIGHT * 4],
    tile_texture: TextureHandle,
    game_info: Option<GameInfo>,
}

impl Debugger {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Debugger {
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
            game_info: None
        }
    }

    pub fn has_game_info(&self) -> bool {
        self.game_info.is_some()
    }

    pub fn load_game_info(&mut self, game_info: GameInfo) {
        self.game_info.replace(game_info);
    }

    pub fn reset(&mut self) {
        self.game_info = None;
    }

    /// Renders the entirety of the debugger window
    pub fn render(&mut self, ui: &mut egui::Ui, debug_info: DebugInfo, tiles: Option<Vec<[u8 ; 16]>>) -> bool {
        let mut stay_open = true;

        if self.vram_debug && let Some(ref t) = tiles { 
            self.draw_vram(t); 
        }

        egui::Panel::right("tiles")
            .exact_size(450.0)
            .show(ui, |ui| {
                ui.heading("VRAM Tiles Visualizer");

                if ui.checkbox(&mut self.vram_debug, "Enable VRAM vizualization").changed() {
                    if self.vram_debug && let Some(ref t) = tiles { 
                        self.draw_vram(t); 
                    }
                }

                if self.vram_debug {
                    ui.vertical_centered(|ui| {
                        let scale = (ui.available_height() / DEBUG_HEIGHT as f32).floor();
                        let image_widget = egui::Image::new(&self.tile_texture)
                            .fit_to_original_size(scale);

                        ui.add(image_widget);
                    });
                }
            }
        );

        egui::Panel::bottom("game_info").show(ui, |ui| {
            ui.heading("Game Information");
            ui.add_space(20.0);

            let game_info = self.game_info.as_ref().unwrap(); // This is safe because the debugger is ever only shown when a game is loaded

            egui::Grid::new("game_data")
                .num_columns(2)
                .min_col_width(ui.available_width() / 2.0)
                .spacing([0.0, 10.0])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(underlined_text("Name"));
                        ui.label(&game_info.name);
                    });

                    let mut sgb_support = game_info.sgb_support;
                    
                    ui.vertical(|ui| {
                        ui.label(underlined_text("Game Type"));
                        ui.horizontal(|ui| {
                            ui.label(game_info.game_type);
                            ui.add_enabled(false, egui::Checkbox::new(&mut sgb_support, "SGB Support"));
                        })
                    });

                    ui.end_row();

                    ui.vertical(|ui| {
                        ui.label(underlined_text("Cartridge Type"));
                        ui.label(&game_info.cartridge_type);
                    });

                    ui.vertical(|ui| {
                        ui.label(underlined_text("ROM Size"));
                        ui.label(format!("{} KiB", game_info.rom_size));
                    });

                    ui.end_row();

                    ui.vertical(|ui| {
                        ui.label(underlined_text("License"));
                        ui.label(&game_info.license);
                    });

                    ui.vertical(|ui| {
                        ui.label(underlined_text("RAM Size"));
                        ui.label(format!("{} KiB", game_info.ram_size));
                    });
                })
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("GameBoy emulator state");

            ui.label("Current Instruction");
            ui.label(debug_info.current_instruction);

            ui.separator();

            if ui.input(|i| i.viewport().close_requested()) {
                // Tell parent to close us.
                stay_open = false;
            }
        });
        stay_open
    }

    fn draw_vram(&mut self, tiles: &Vec<[u8 ; 16]>) {
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

fn underlined_text(text: impl Into<String>) -> RichText {
    egui::RichText::new(text)
        .underline()
        .weak()
}