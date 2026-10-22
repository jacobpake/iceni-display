use crate::display::volatile_cell::VolatileCell;

unsafe extern "Rust" {
    #[cheriot_mmio(name = "gpio", permissions = "RW")]
    pub static GPIO: GpioPin;
}

#[allow(unused)]
pub enum Pin {
    CS = 0x13,    // PB03
    CLK = 0x14,   // PB04
    MOSI = 0x15,  // PB05
    MISO = 0x16,  // PB06
    SDLCD = 0x17, // PB07 - sd or lcd
    LED = 0x18,   // PB08 - backlight
    DC = 0x1b,    // PB11 - data/command
    RST = 0x1c,   // PB12 - reset
}

pub struct Gpio(*mut GpioPin);

impl Gpio {
    pub unsafe fn get() -> Self {
        Self((&raw const GPIO) as *mut GpioPin)
    }
}

impl core::ops::Index<Pin> for Gpio {
    type Output = GpioPin;

    fn index(&self, index: Pin) -> &GpioPin {
        unsafe { &*self.0.add(index as usize) }
    }
}

#[repr(C)]
pub struct GpioPin {
    pub control: VolatileCell<u32>,
    pub data: VolatileCell<u32>,
}

impl GpioPin {
    pub fn set_low(&self) {
        // IceniGpioPin::Data().with<IceniGpioPin::Data::Output>(0);
        self.data.write(0b00);
    }

    pub fn set_high(&self) {
        // IceniGpioPin::Data().with<IceniGpioPin::Data::Output>(1);
        self.data.write(0b10);
    }

    pub fn function0(&self) {
        // IceniGpioPin::Control()
        //     .with<IceniGpioPin::Control::PinDrive>(
        //         IceniGpioPin::Control::PinDrive::PushPull)
        //     .with<IceniGpioPin::Control::Function>({0});
        self.control.write(0b0000_0011);
    }

    pub fn function1(&self) {
        // IceniGpioPin::Control()
        //     .with<IceniGpioPin::Control::PinDrive>(
        //         IceniGpioPin::Control::PinDrive::PushPull)
        //     .with<IceniGpioPin::Control::Function>({1});
        self.control.write(0b0100_0011);
    }
}
