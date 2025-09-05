use std::{error::Error, fs};

mod header;
use header::CartridgeHeader;

mod rom;
use rom::ROM;

mod mbc1;
use mbc1::MBC1;

trait CartridgeInternals {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn need_save(&mut self) -> bool;
    fn save(&self, save_path: &str);
    fn load_save(&mut self, save_path: &str);
}

pub struct Cartridge {
    _filename: String,
    _rom_size: u32,
    _header: CartridgeHeader,
    cart_internals: Box<dyn CartridgeInternals + Send>,
}

impl Cartridge {
    pub fn load(path: &str) -> Result<Cartridge, Box<dyn Error>> {
        let rom_data = fs::read(path)?;
        let rom_size = (rom_data.len() * 8) as u32;

        let header = CartridgeHeader::from_bytes(&rom_data)?;

        let cart_internals: Box<dyn CartridgeInternals + Send> = match header.cart_type {
            0 => Box::new(ROM::new(rom_data)),
            1..4 => Box::new(MBC1::new(&header, rom_data)),
            _ => unreachable!()
        };

        // println!("Cartridge successfully loaded");
        // println!("\t Title    : {}", header.title);
        // println!(
        //     "\t Type     : {0} ({1})",
        //     header.cart_type,
        //     header.get_cart_type()
        // );
        // println!("\t ROM Size : {} KiB", 32 << header.rom_size);
        // println!("\t RAM Size : {}", header.ram_size);
        // println!(
        //     "\t LIC Code : {0} ({1})",
        //     if header.lic_code != 0x33 {
        //         header.lic_code.to_string()
        //     } else {
        //         str::from_utf8(&header.new_lic_code).unwrap().to_string()
        //     },
        //     header.get_lic_name()
        // );
        // println!("\t ROM Vers : {}", header.version);

        Ok(Cartridge {
            _filename: path.to_string(),
            _rom_size: rom_size,
            _header: header,
            cart_internals,
        })
    }

    pub fn read(&self, address: u16) -> u8 {
        self.cart_internals.read(address)
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.cart_internals.write(address, value);
    }

    pub fn save(&self, save_path: &str) {
        self.cart_internals.save(save_path);
    }

    pub fn load_save(&mut self, save_path: &str) {
        self.cart_internals.load_save(save_path);
    }

    pub fn need_save(&mut self) -> bool {
        self.cart_internals.need_save()
    }
}
