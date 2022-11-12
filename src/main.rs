#![no_std]
#![no_main]

extern crate panic_halt;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::InputPin;
use hd44780_driver::{bus::FourBitBus, Cursor, CursorBlink, Display, DisplayMode, HD44780};
use keypad2::Keypad;
use stm32f4xx_hal::{
    gpio::{
        gpioa::{PA10, PA2, PA3},
        gpiob::{PB10, PB3, PB4, PB5},
        Input, OpenDrain, Output, Pin, Pull,
    },
    pac::{self, TIM1},
    prelude::*,
    timer::Delay,
};

// Connections:
// GND: GND
// VDD: 5V
// V0:  10k poti between 5V and GND
// RS:  D9 / PC7
// RW:  GND
// E:   D10 / PB6
// D4:  D11 / PA7
// D5:  D12 / PA6
// D6:  D7 / PA8
// D7:  D6 / PB10
// A:   5V
// K:   GND

// Keypad connections:
// from left to right:
// D0 / PA3 (C2)
// D1 / PA2 (R1)
// D2 / PA10 (C1)
// D3 / PB3 (R4)
// D4 / PB5 (C3)
// D5 / PB4 (R3)
// discon / D6 / PB10 (R2)

// max chars in display
const MAX_DISPLAY_CHARS: usize = 16;

type GenericKeypad = Keypad<
    Pin<'A', 2>,
    Pin<'B', 10>,
    Pin<'B', 4>,
    Pin<'B', 3>,
    Pin<'A', 10, Output<OpenDrain>>,
    Pin<'A', 3, Output<OpenDrain>>,
    Pin<'B', 5, Output<OpenDrain>>,
>;

type GenericDelay = Delay<TIM1, 1000000>;

type GenericDisplay = HD44780<
    FourBitBus<
        Pin<'C', 7, Output>,
        Pin<'B', 6, Output>,
        Pin<'A', 7, Output>,
        Pin<'A', 6, Output>,
        Pin<'A', 8, Output>,
        Pin<'B', 0, Output>,
    >,
>;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let gpiob = dp.GPIOB.split();
    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();

    let clocks = rcc.cfgr.freeze();
    let mut delay = dp.TIM1.delay_us(&clocks);

    let dummy_pin = gpiob.pb0.into_push_pull_output();

    let rows = (
        gpioa.pa2.into_pull_up_input(),
        gpiob.pb10.into_pull_up_input(),
        gpiob.pb4.into_pull_up_input(),
        gpiob.pb3.into_pull_up_input(),
    );
    let cols = (
        gpioa.pa10.into_open_drain_output(),
        gpioa.pa3.into_open_drain_output(),
        gpiob.pb5.into_open_drain_output(),
    );

    let mut keypad = Keypad::new(rows, cols);

    let rs = gpioc.pc7.into_push_pull_output();
    let en = gpiob.pb6.into_push_pull_output();
    let d4 = gpioa.pa7.into_push_pull_output();
    let d5 = gpioa.pa6.into_push_pull_output();
    let d6 = gpioa.pa8.into_push_pull_output();
    let d7 = dummy_pin;

    let mut lcd = HD44780::new_4bit(rs, en, d4, d5, d6, d7, &mut delay).unwrap();
    lcd.reset(&mut delay).unwrap();
    lcd.clear(&mut delay).unwrap();
    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Visible,
            cursor_blink: CursorBlink::On,
        },
        &mut delay,
    )
    .unwrap();
    lcd.write_str("Booting...", &mut delay).unwrap();
    lcd.set_cursor_pos(40, &mut delay).unwrap();
    lcd.write_str("Num2", &mut delay).unwrap();

    let mut led = gpioa.pa5.into_push_pull_output();
    #[allow(clippy::empty_loop)]
    loop {
        delay.delay_ms(500_u16);

        let key = keypad.read_char(&mut delay);

        if key != ' ' {
            if key == '*' || key == '#' {
                continue;
            }
            //lcd.reset(&mut delay).unwrap();
            //lcd.write_char(key, &mut delay).unwrap();

            let a = key.to_digit(10).unwrap();
            for i in 0..a {
                led.set_high();
                delay.delay_ms(300u32);
                led.set_low();
                delay.delay_ms(300u32);
            }
        }

        delay.delay_ms(1u16);
    }
}

fn read_char(keypad: &mut GenericKeypad, delay: &mut GenericDelay) -> char {
    delay.delay_ms(1_u16);

    loop {
        let key = keypad.read_char(delay);

        if key != ' ' {
            if key == '#' {
                // treat as enter
                return ' ';
            } else if key == '*' {
                return '.';
            } else {
                // number
                return key;
            }
        }
        delay.delay_ms(10u16);
    }
}

fn read_line(string: &mut [char], keypad: &mut GenericKeypad, delay: &mut GenericDelay) {
    // TODO: display text on screen on input
    delay.delay_ms(1_u16);
    let mut index = 0;
    loop {
        let key = keypad.read_char(delay);

        if key != ' ' {
            let mut char = '.';
            if key == '#' {
                // treat as enter
                break;
            } else if key == '*' {
                // decimal point
                char = '.';
            } else {
                // number
                char = key;
            }

            // make sure we don't overflow display
            if index != MAX_DISPLAY_CHARS {
                string[index] = char;
                index += 1;
            }
        }
        delay.delay_ms(10u16);
    }
    for i in index..MAX_DISPLAY_CHARS {
        string[i] = '\0';
    }
}

fn write_screen(first: &str, second: &str, lcd: &mut GenericDisplay, delay: &mut GenericDelay) {
    delay.delay_ms(10u16);
    lcd.reset(delay).unwrap();
    lcd.write_str(first, delay).unwrap();
    lcd.set_cursor_pos(40u8, delay).unwrap();
    lcd.write_str(second, delay).unwrap();
}

fn write_line(string: &str, second_line: bool, lcd: &mut GenericDisplay, delay: &mut GenericDelay) {
    delay.delay_ms(10u16);

    let pos = if second_line { 40u8 } else { 0u8 };
    lcd.set_cursor_pos(pos, delay).unwrap();
    lcd.write_str(string, delay).unwrap(); // hope it also clears the rest of the line
}
