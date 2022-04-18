#![no_std]
#![no_main]

use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::{
    mono_font::ascii::FONT_4X6,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_time::fixed_point::FixedPoint as _;
use hal::{i2c::I2c, timer::MonoTimer};
use pac::{CorePeripherals, Peripherals};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use stm32f3xx_hal::prelude::_embedded_hal_digital_InputPin;
use stm32f3xx_hal::{self as hal, pac, prelude::*};

const IMAGE_WIDTH: usize = 21;
const IMAGE_HEIGHT: usize = 10;
const TOTAL_FRAMES: usize = 1749;
const DRAW_TIME: usize = 125; // ms for a single screen draw

const IMAGE_LEN: usize = IMAGE_HEIGHT * (IMAGE_WIDTH + 1); // adjust for newline char
const IMAGE_END: usize = TOTAL_FRAMES * IMAGE_LEN; // position of the last ascii char

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let peripherals = Peripherals::take().unwrap();
    let mut core_peripherals = CorePeripherals::take().unwrap();

    let mut rcc = peripherals.RCC.constrain();
    let mut flash = peripherals.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb);
    let mut gpioc = peripherals.GPIOC.split(&mut rcc.ahb);
    let monotimer = MonoTimer::new(core_peripherals.DWT, clocks, &mut core_peripherals.DCB);
    let mut delay_provider = Delay::new(core_peripherals.SYST, clocks.hclk().integer());

    let button1 = gpioc
        .pc13
        .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr);

    let scl = gpiob
        .pb8
        .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrh);
    let sda = gpiob
        .pb9
        .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrh);

    let i2c = I2c::new(
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

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_4X6)
        .text_color(BinaryColor::On)
        .build();

    let ascii_txt = core::str::from_utf8(include_bytes!("../assets/ascii.txt")).unwrap();

    // start indexing at 0, draw IMAGE_LEN ascii chars to display
    let mut index: usize = 0;
    loop {
        let monotimer_instant = monotimer.now();

        // reset
        if button1.is_high().unwrap() {
            index = 0
        }

        display.clear();

        let text = Text::with_baseline(
            &ascii_txt[index..index + IMAGE_LEN],
            Point::new(0, 0),
            text_style,
            Baseline::Top,
        );
        text.draw(&mut display).unwrap();
        // draw out to physical screen
        display.flush().unwrap();

        // go to next frame or reset to start
        index = (index + IMAGE_LEN) % IMAGE_END;

        // adjust for desired framerate
        // WARNING: don't do anything after elapsed() has been calculated,
        // otherwise it will delay the frame being drawn, which will
        // knock it off sync
        // It's still a tiny, wheensy bit off, but it's fine
        let freq: f64 = monotimer.frequency().0.into();
        let ms_per_draw: f64 = f64::from(1000 ) / (freq / f64::from(monotimer_instant.elapsed()));
        match DRAW_TIME.checked_sub(ms_per_draw as usize) {
            Some(result) => delay_provider.delay_ms(result as u32),
            None => (),
        };
    }
}
