use std::{error::Error, fmt, fs, io};

const ROM_TYPES: [&str; 0x23] = [
    "ROM ONLY",
    "MBC1",
    "MBC1+RAM",
    "MBC1+RAM+BATTERY",
    "0x04 ???",
    "MBC2",
    "MBC2+BATTERY",
    "0x07 ???",
    "ROM+RAM 1",
    "ROM+RAM+BATTERY 1",
    "0x0A ???",
    "MMM01",
    "MMM01+RAM",
    "MMM01+RAM+BATTERY",
    "0x0E ???",
    "MBC3+TIMER+BATTERY",
    "MBC3+TIMER+RAM+BATTERY 2",
    "MBC3",
    "MBC3+RAM 2",
    "MBC3+RAM+BATTERY 2",
    "0x14 ???",
    "0x15 ???",
    "0x16 ???",
    "0x17 ???",
    "0x18 ???",
    "MBC5",
    "MBC5+RAM",
    "MBC5+RAM+BATTERY",
    "MBC5+RUMBLE",
    "MBC5+RUMBLE+RAM",
    "MBC5+RUMBLE+RAM+BATTERY",
    "0x1F ???",
    "MBC6",
    "0x21 ???",
    "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
];

static LIC_CODES: [Option<&'static str>; 0xA5] = {
    let mut codes = [None; 0xA5];
    codes[0x00] = Some("None");
    codes[0x01] = Some("Nintendo R&D1");
    codes[0x08] = Some("Capcom");
    codes[0x13] = Some("Electronic Arts");
    codes[0x18] = Some("Hudson Soft");
    codes[0x19] = Some("b-ai");
    codes[0x20] = Some("kss");
    codes[0x22] = Some("pow");
    codes[0x24] = Some("PCM Complete");
    codes[0x25] = Some("san-x");
    codes[0x28] = Some("Kemco Japan");
    codes[0x29] = Some("seta");
    codes[0x30] = Some("Viacom");
    codes[0x31] = Some("Nintendo");
    codes[0x32] = Some("Bandai");
    codes[0x33] = Some("Ocean/Acclaim");
    codes[0x34] = Some("Konami");
    codes[0x35] = Some("Hector");
    codes[0x37] = Some("Taito");
    codes[0x38] = Some("Hudson");
    codes[0x39] = Some("Banpresto");
    codes[0x41] = Some("Ubi Soft");
    codes[0x42] = Some("Atlus");
    codes[0x44] = Some("Malibu");
    codes[0x46] = Some("angel");
    codes[0x47] = Some("Bullet-Proof");
    codes[0x49] = Some("irem");
    codes[0x50] = Some("Absolute");
    codes[0x51] = Some("Acclaim");
    codes[0x52] = Some("Activision");
    codes[0x53] = Some("American sammy");
    codes[0x54] = Some("Konami");
    codes[0x55] = Some("Hi tech entertainment");
    codes[0x56] = Some("LJN");
    codes[0x57] = Some("Matchbox");
    codes[0x58] = Some("Mattel");
    codes[0x59] = Some("Milton Bradley");
    codes[0x60] = Some("Titus");
    codes[0x61] = Some("Virgin");
    codes[0x64] = Some("LucasArts");
    codes[0x67] = Some("Ocean");
    codes[0x69] = Some("Electronic Arts");
    codes[0x70] = Some("Infogrames");
    codes[0x71] = Some("Interplay");
    codes[0x72] = Some("Broderbund");
    codes[0x73] = Some("sculptured");
    codes[0x75] = Some("sci");
    codes[0x78] = Some("THQ");
    codes[0x79] = Some("Accolade");
    codes[0x80] = Some("misawa");
    codes[0x83] = Some("lozc");
    codes[0x86] = Some("Tokuma Shoten Intermedia");
    codes[0x87] = Some("Tsukuda Original");
    codes[0x91] = Some("Chunsoft");
    codes[0x92] = Some("Video system");
    codes[0x93] = Some("Ocean/Acclaim");
    codes[0x95] = Some("Varie");
    codes[0x96] = Some("Yonezawa/s’pal");
    codes[0x97] = Some("Kaneko");
    codes[0x99] = Some("Pack in soft");
    codes[0xA4] = Some("Konami (Yu-Gi-Oh!)");
    codes
};


struct CartridgeHeader {
    entry: [u8; 4],
    logo: [u8; 0x30],
    title: [u8; 16],
    new_lic_code: u16,
    sgb_flag: u8,
    cart_type: u8,
    rom_size: u8,
    ram_size: u8,
    dest_code: u8,
    lic_code: u8,
    version: u8,
    checksum: u8,
    global_checksum: u16,
}

impl CartridgeHeader {
    fn from_bytes(data: &[u8]) -> Result<CartridgeHeader, InvalidCartridge> {
        if data.len() < 0x150 {
            eprintln!("Not enough data!");
            return Err(InvalidCartridge {  });
        }

        let mut checksum: u8 = 0;
        for i in 0x0134..=0x014C {
            checksum = checksum.wrapping_sub(data[i] + 1)
        }

        if (checksum & 0xFF) != data[0x14D] {
            eprintln!("Incorrect checksum!");
            return Err(InvalidCartridge {  })
        }

        Ok(Self {
            entry: data[0x100..0x104].try_into().unwrap(),
            logo: data[0x104..0x134].try_into().unwrap(),
            title: data[0x134..0x144].try_into().unwrap(),
            new_lic_code: u16::from_be_bytes(data[0x144..0x146].try_into().unwrap()),
            sgb_flag: data[0x146],
            cart_type: data[0x147],
            rom_size: data[0x148],
            ram_size: data[0x149],
            dest_code: data[0x14A],
            lic_code: data[0x14B],
            version: data[0x14C],
            checksum,
            global_checksum: u16::from_be_bytes(data[0x14E..0x150].try_into().unwrap()),
        })
    }
}

pub struct Cartridge {
    filename: String,
    rom_size: u32,
    rom_data: Vec<u8>,
    header: CartridgeHeader
}

impl Cartridge {
    pub fn load(path: &str) -> Result<Cartridge, Box<dyn Error>> {
        let rom_data = fs::read(path)?;
        let rom_size = (rom_data.len() * 8) as u32;

        let header = CartridgeHeader::from_bytes(&rom_data)?;

        Ok(Cartridge {
            filename: path.to_string(),
            rom_size,
            rom_data,
            header
        })

    }

    fn get_lic_name(&self) -> &str {
        let lic_code = self.header.lic_code as usize;
        if lic_code >= LIC_CODES.len() {
            return "UNKNOWN"
        }
        match LIC_CODES[lic_code] {
            Some(lic) => lic,
            None => "UNKNOWN"
        }
    }

    fn get_cart_type(&self) -> &str{
        let cart_type = self.header.cart_type as usize;
        if cart_type >= ROM_TYPES.len() {
            return "UNKNOWN"
        } else {
            return ROM_TYPES[cart_type]
        }
    }
}

/// This error is returned when the reading has succeeded 
/// but the cartridge is invalid
#[derive(Debug)]
struct InvalidCartridge {}

impl fmt::Display for InvalidCartridge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid cartridge loaded")
    }
}

impl Error for InvalidCartridge {}