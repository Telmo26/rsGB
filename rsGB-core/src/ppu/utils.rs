use crate::interconnect::Interconnect;

pub enum LCDMode {
    HBlank = 0,
    VBlank = 1,
    OAM = 2,
    XFer = 3,
}

pub enum StatusSrc {
    HBlank = (1 << 3),
    VBlank = (1 << 4),
    OAM = (1 << 5),
    LYC = (1 << 6),
}

pub fn lcd_read_ly(bus: &mut Interconnect) -> u8 { bus.read(0xFF44) }

pub fn lcd_write_ly(bus: &mut Interconnect, value: u8) { bus.write(0xFF44, value) }

pub fn lcd_read_scroll_x(bus: &mut Interconnect) -> u8 { bus.read(0xFF43) }

pub fn lcd_read_scroll_y(bus: &mut Interconnect) -> u8 { bus.read(0xFF42) }

pub fn change_lcd_mode(bus: &mut Interconnect, mode: LCDMode) {
    let mut status = bus.read(0xFF41);
    status &= !0b11;
    status |= mode as u8;
    bus.write(0xFF41, status);
}

pub fn lcdc_bgw_enable(bus: &mut Interconnect) -> bool { (bus.read(0xFF40) & 1) != 0 }

pub fn lcdc_obj_enable(bus: &mut Interconnect) -> bool { (bus.read(0xFF40) & (1 << 1)) != 0 }

pub fn lcdc_obj_height(bus: &mut Interconnect) -> u8 {
    if bus.read(0xFF40) & (1 << 2) != 0 {
        16
    } else {
        8
    }
}

pub fn lcdc_bg_map_area(bus: &mut Interconnect) -> u16 {
    if bus.read(0xFF40) & (1 << 3) != 0 {
        0x9C00
    } else {
        0x9800
    }
}

pub fn lcdc_bgw_data_area(bus: &mut Interconnect) -> u16 {
    if bus.read(0xFF40) & (1 << 4) != 0 {
        0x8000
    } else {
        0x8800
    }
}

pub fn lcdc_win_enable(bus: &mut Interconnect) -> bool { (bus.read(0xFF40) & (1 << 5)) != 0 }

fn lcdc_win_map_area(bus: &mut Interconnect) -> u16 {
    if bus.read(0xFF40) & (1 << 6) != 0 {
        0x9800
    } else {
        0x9C00
    }
}

pub fn lcdc_lcd_enable(bus: &mut Interconnect) -> bool { (bus.read(0xFF40) & (1 << 7)) != 0 }

pub fn status_mode(bus: &mut Interconnect) -> LCDMode {
    match bus.read(0xFF41) & 0b11 {
        0 => LCDMode::HBlank,
        1 => LCDMode::VBlank,
        2 => LCDMode::OAM,
        _ => LCDMode::XFer,
    }
}

pub fn status_mode_set(bus: &mut Interconnect, mode: LCDMode) {
    let mut status = bus.read(0xFF41) & !0b11;
    match mode {
        LCDMode::HBlank => status |= 0b00,
        LCDMode::VBlank => status |= 0b01,
        LCDMode::OAM => status |= 0b10,
        LCDMode::XFer => status |= 0b11, 
    }
    bus.write(0xFF41, status)
}

pub fn status_lyc(bus: &mut Interconnect) -> bool { bus.read(0xFF41) & (1 << 2) != 0 }

pub fn status_lyc_set(bus: &mut Interconnect, value: u8) {
    let status = bus.read(0xFF41) | (value << 2);
    bus.write(0xFF41, status) 
} 

pub fn status_stat_int(bus: &mut Interconnect, src: StatusSrc) -> bool { bus.read(0xFF41) & (src as u8) != 0 }