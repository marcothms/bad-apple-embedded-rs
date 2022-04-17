#![no_std]
#![no_main]

use cortex_m::prelude::*;
use cortex_m_rt::entry;
use embedded_graphics::{
    mono_font::ascii::FONT_4X6,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use stm32f3xx_hal::{self as hal, pac, prelude::*};
use stm32f3xx_hal::prelude::_embedded_hal_digital_InputPin;

const IMAGE_LEN: usize = 220; // size of a single ascii image on screen
const IMAGE_END: usize = 1093 * 220; // position of the last ascii char

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let peripherals = pac::Peripherals::take().unwrap();
    let mut core_peripherals = pac::CorePeripherals::take().unwrap();

    let mut rcc = peripherals.RCC.constrain();
    let mut flash = peripherals.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb);
    let mut gpioc = peripherals.GPIOC.split(&mut rcc.ahb);
    let monotimer = hal::timer::MonoTimer::new(
        core_peripherals.DWT,
        clocks,
        &mut core_peripherals.DCB);
    // TODO: dont use hardcoded val here
    let mut delay = cortex_m::delay::Delay::new(core_peripherals.SYST, 8000000);

    let mut button1 = gpioc
        .pc13
        .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr);

    let scl = gpiob
        .pb8
        .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrh);
    let sda = gpiob
        .pb9
        .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrh);

    let i2c = hal::i2c::I2c::new(
        peripherals.I2C1,
        (scl, sda),
        1000.kHz().try_into().unwrap(),
        clocks,
        &mut rcc.apb1,
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = embedded_graphics::mono_font::MonoTextStyleBuilder::new()
        .font(&FONT_4X6)
        .text_color(BinaryColor::On)
        .build();

    let bad_apple = core::str::from_utf8(include_bytes!("../assets/gen.txt")).unwrap();

    // start indexing at 0, draw IMAGE_LEN ascii chars to display
    let mut index: usize = 0;
    loop {
        let instant = monotimer.now();
        display.clear();

        rprintln!("draw: {} to {}", index, index + IMAGE_LEN);
        let text = Text::with_baseline(
            &bad_apple[index..index + IMAGE_LEN],
            Point::new(0, 0),
            text_style,
            Baseline::Top,
        );
        text.draw(&mut display).unwrap();
        display.flush().unwrap();

        // go to next frame or reset to start
        index = (index + IMAGE_LEN) % IMAGE_END;

        let elapsed = instant.elapsed();
        rprintln!("draw took: {}", elapsed);
        rprintln!("freq: {}", monotimer.frequency());
        delay.delay_ms(100);

        // kill switch
        if button1.is_high().unwrap() {
            loop {}
        }
    }
}
