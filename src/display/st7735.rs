// use crate::display::BMP_WIDTH;
use crate::display::gpio::{Gpio, Pin};
use crate::display::spi::Spi;

unsafe extern "C" {
    fn sleep(ms: u32);
}

#[allow(unused)]
pub enum Instruction {
    NOP = 0x00,
    SWRESET = 0x01,
    RDDID = 0x04,
    RDDST = 0x09,
    SLPIN = 0x10,
    SLPOUT = 0x11,
    PTLON = 0x12,
    NORON = 0x13,
    INVOFF = 0x20,
    INVON = 0x21,
    DISPOFF = 0x28,
    DISPON = 0x29,
    CASET = 0x2A,
    RASET = 0x2B,
    RAMWR = 0x2C,
    RAMRD = 0x2E,
    PTLAR = 0x30,
    COLMOD = 0x3A,
    MADCTL = 0x36,
    FRMCTR1 = 0xB1,
    FRMCTR2 = 0xB2,
    FRMCTR3 = 0xB3,
    INVCTR = 0xB4,
    DISSET5 = 0xB6,
    PWCTR1 = 0xC0,
    PWCTR2 = 0xC1,
    PWCTR3 = 0xC2,
    PWCTR4 = 0xC3,
    PWCTR5 = 0xC4,
    VMCTR1 = 0xC5,
    RDID1 = 0xDA,
    RDID2 = 0xDB,
    RDID3 = 0xDC,
    RDID4 = 0xDD,
    PWCTR6 = 0xFC,
    GMCTRP1 = 0xE0,
    GMCTRN1 = 0xE1,
}

pub struct ST7735 {
    spi: Spi,
    gpio: Gpio,
}

#[allow(unused)]
pub enum Orientation {
    Portrait = 0x00,
    Landscape = 0x60,
    PortraitSwapped = 0xC0,
    LandscapeSwapped = 0xA0,
}

impl ST7735 {
    /// Creates a new driver instance that uses hardware SPI.
    pub fn new(spi: Spi, gpio: Gpio) -> Self {
        return ST7735 { spi, gpio };
    }

    pub fn draw_pixel(&mut self, px: u16, py: u16, rgb: u16) {
        self.draw_rect(px, py, px, py, rgb);
    }

    pub fn draw_rect(&mut self, sx: u16, sy: u16, ex: u16, ey: u16, rgb: u16) {
        self.set_address_window(sx, sy, ex, ey);
        self.write_command_params(Instruction::RAMWR, &[]);
        self.gpio[Pin::DC].set_high();
        self.spi.write(&rgb.to_be_bytes());
    }

    /// Runs commands to initialize the display.
    pub fn init(&mut self) {
        self.hard_reset();

        // init_script_r
        self.write_command_params(Instruction::SWRESET, &[]);
        unsafe { sleep(150) };
        self.write_command_params(Instruction::SLPOUT, &[]);
        unsafe { sleep(500) };
        self.write_command_params(Instruction::FRMCTR1, &[0x00, 0x02, 0x02]);
        self.write_command_params(Instruction::FRMCTR2, &[0x00, 0x02, 0x02]);
        self.write_command_params(Instruction::FRMCTR3, &[0x00, 0x02, 0x02, 0x00, 0x02, 0x02]);
        self.write_command_params(Instruction::INVCTR, &[0x07]);
        self.write_command_params(Instruction::PWCTR1, &[0xA2, 0x02, 0x84]);
        self.write_command_params(Instruction::PWCTR2, &[0xC5]);
        self.write_command_params(Instruction::PWCTR3, &[0x0A, 0x00]);
        self.write_command_params(Instruction::PWCTR4, &[0x8A, 0x2A]);
        self.write_command_params(Instruction::PWCTR5, &[0x8A, 0xEE]);
        self.write_command_params(Instruction::VMCTR1, &[0x0E]);
        self.write_command_params(Instruction::INVOFF, &[]);
        // i cant remember why this is commented out
        // self.write_command_params(Instruction::MADCTL, &[0x08]);
        self.write_command_params(Instruction::COLMOD, &[0x05]);

        // init_script_r2

        self.write_command_params(Instruction::CASET, &[0x00, 0x02, 0x00, 0x81]); // 0x7F + 0x02
        self.write_command_params(Instruction::RASET, &[0x00, 0x01, 0x00, 0xA0]); // 0x9F + 0x01

        // init_script_r3
        self.write_command_params(
            Instruction::GMCTRP1,
            &[
                0x02, 0x1c, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2d, 0x29, 0x25, 0x2B, 0x39, 0x00, 0x01,
                0x03, 0x10,
            ],
        );
        self.write_command_params(
            Instruction::GMCTRN1,
            &[
                0x03, 0x1d, 0x07, 0x06, 0x2E, 0x2C, 0x29, 0x2D, 0x2E, 0x2E, 0x37, 0x3F, 0x00, 0x00,
                0x02, 0x10,
            ],
        );
        self.write_command_params(Instruction::NORON, &[]);
        unsafe { sleep(10) };
        self.write_command_params(Instruction::DISPON, &[]);
        unsafe { sleep(100) };

        self.set_orientation(Orientation::LandscapeSwapped);
    }

    pub fn hard_reset(&mut self) {
        self.gpio[Pin::RST].set_low();
        unsafe { sleep(100) };
        self.gpio[Pin::RST].set_high();
    }

    #[inline(always)]
    fn write_command_params(&mut self, command: Instruction, params: &[u8]) {
        self.gpio[Pin::DC].set_low();
        self.spi.write(&[command as u8]);
        if !params.is_empty() {
            self.gpio[Pin::DC].set_high();
            self.spi.write(params);
        }
    }

    /// Writes a data word to the display.
    #[inline(always)]
    fn write_word(&mut self, value: u16) {
        self.spi.write(&value.to_be_bytes());
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.write_command_params(Instruction::MADCTL, &[orientation as u8]);
    }

    /// Sets the address window for the display.
    pub fn set_address_window(&mut self, sx: u16, sy: u16, ex: u16, ey: u16) {
        self.write_command_params(Instruction::CASET, &[]);
        self.gpio[Pin::DC].set_high();
        self.write_word(sx);
        self.write_word(ex);
        self.write_command_params(Instruction::RASET, &[]);
        self.gpio[Pin::DC].set_high();
        self.write_word(sy);
        self.write_word(ey);
    }
}
