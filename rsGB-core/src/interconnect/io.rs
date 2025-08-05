mod timer;
mod dma;
mod lcd;
mod gamepad;

use timer::Timer;
use dma::DMA;
use lcd::LCD;
use gamepad::Gamepad;

use super::InterruptType;

/*

Start | End   | 1st app | Purpose
---------------------------------------------------------------------------
$FF00 | 	  |   DMG 	| Joypad input
$FF01 | $FF02 |   DMG 	| Serial transfer
$FF04 | $FF07 |   DMG 	| Timer and divider
$FF0F | 	  |   DMG 	| Interrupts
$FF10 | $FF26 |   DMG 	| Audio
$FF30 | $FF3F |   DMG 	| Wave pattern
$FF40 | $FF4B |   DMG 	| LCD Control, Status, Position, Scrolling, and Palettes
$FF4F | 	  |   CGB 	| VRAM Bank Select
$FF50 | 	  |   DMG 	| Boot ROM mapping control
$FF51 | $FF55 |   CGB 	| VRAM DMA
$FF68 | $FF6B |   CGB 	| BG / OBJ Palettes
$FF70 | 	  |   CGB 	| WRAM Bank Select

*/


pub struct IO {
    gamepad: Gamepad,
    serial: [u8; 2],
    timer: Timer,
    if_register: u8,
    pub(crate) lcd: LCD,
    dma: DMA,
}

impl IO {
    pub fn new() -> IO {
        IO { 
            gamepad: Gamepad::new(),
            serial: [0; 2],
            timer: Timer::new(),
            if_register: 0,
            lcd: LCD::new(),
            dma: DMA::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF00 => self.gamepad.get_output(),
            0xFF01 => self.serial[0],
            0xFF02 => self.serial[1],
            0xFF04..=0xFF07 => self.timer.read(address),
            0xFF0F => self.if_register,
            0xFF40..=0xFF4B => self.lcd.read(address),
            _ => {
                eprintln!("Read at address {address:X} not implemented!");
                0
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF00 => self.gamepad.set_sel(value),
            0xFF01 => self.serial[0] = value,
            0xFF02 => self.serial[1] = value,
            0xFF04..=0xFF07 => self.timer.write(address, value),
            0xFF0F => self.if_register = value,
            0xFF40..=0xFF4B => {
                if address == 0xFF46 { self.dma.start(value) }
                self.lcd.write(address, value);
            }
            _ => eprintln!("Write at address {address:X} not implemented!"),
        }
    }

    pub fn tick_timer(&mut self) {
        let interrupt = self.timer.tick();
        if let Some(interrupt) = interrupt {
            self.if_register |= interrupt as u8;
        }
    }

    pub fn request_interrupt(&mut self, interrupt: InterruptType) {
        self.if_register |= interrupt as u8;
    }

    pub fn tick_dma(&mut self) -> Option<(u8, u8)> {
        self.dma.tick()
    }

    pub fn dma_transferring(&self) -> bool {
        self.dma.transferring()
    }
}