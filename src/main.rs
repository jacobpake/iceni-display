#![feature(cheriot_attributes)]

mod display;

use display::{
    gpio::{Gpio, Pin},
    spi::Spi,
    st7735::ST7735,
};

fn gpio_init() -> Gpio {
    let gpio = unsafe { Gpio::get() };
    gpio[Pin::CS].function1(); // spi is 1
    gpio[Pin::CLK].function1(); // spi is 1
    gpio[Pin::MOSI].function1(); // spi is 1
    gpio[Pin::MISO].function1(); // spi is 1

    gpio[Pin::DC].function1(); // gpio is 1
    gpio[Pin::RST].function1(); // gpio is 1
    gpio[Pin::LED].function0(); // gpio is 0
    gpio[Pin::SDLCD].function0(); // gpio is 0

    gpio[Pin::SDLCD].set_low(); // select LCD (not SD)
    gpio[Pin::LED].set_high(); // backlight on

    gpio
}

fn spi_init() -> Spi {
    let spi = unsafe { Spi::get() };
    let spi_init = spi.init(1, 0, 0, 0, false, true);

    if let Err(spi_init) = spi_init {
        panic!("panic!!! {spi_init:?}");
    }

    spi
}

fn main() {
    let gpio = gpio_init();
    let spi = spi_init();

    let mut display = ST7735::new(spi, gpio);

    display.init();

    let mut colour = 0_u16;

    display.draw_rect(0, 0, 159, 119, 0xffbb);

    loop {
        for x in 0..160 {
            for y in 0..120 {
                display.draw_pixel(x, y, colour);
                colour += 1;
            }
        }
    }
}
