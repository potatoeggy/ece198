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
mod calcs;

const MAX_DISPLAY_CHARS: usize = 16;

pub type GenericKeypad = Keypad<
    Pin<'A', 2>,
    Pin<'B', 10>,
    Pin<'B', 4>,
    Pin<'B', 3>,
    Pin<'A', 10, Output<OpenDrain>>,
    Pin<'A', 3, Output<OpenDrain>>,
    Pin<'B', 5, Output<OpenDrain>>,
>;

pub type GenericDelay = Delay<TIM1, 1000000>;

pub type GenericDisplay = HD44780<
    FourBitBus<
        Pin<'C', 7, Output>,
        Pin<'B', 6, Output>,
        Pin<'A', 7, Output>,
        Pin<'A', 6, Output>,
        Pin<'A', 8, Output>,
        Pin<'B', 0, Output>,
    >,
>;

pub struct WaterData {
    ph: f64,
    cond: f64,
    hardness: f64,
}

impl WaterData {
    pub fn new() -> WaterData {
        WaterData {
            ph: 0.0,
            cond: 0.0,
            hardness: 0.0,
        }
    }
}

fn print_main_menu(lcd: &mut GenericDisplay, delay: &mut GenericDelay) {
    write_screen("1. New data", "2. Summary (x)", lcd, delay);
}

fn add_data(
    keypad: &mut GenericKeypad,
    lcd: &mut GenericDisplay,
    delay: &mut GenericDelay,
) -> WaterData {
    let mut ph = 0.0;
    let mut cond = 0.0;
    let mut hard = 0.0;

    let prompts = [
        ("pH:", &mut ph),
        ("Conduc. (mS/cm):", &mut cond),
        ("Hardness (mg/L):", &mut hard),
    ];

    for (text, var) in prompts {
        lcd.reset(delay).unwrap();
        lcd.clear(delay).unwrap();
        write_line(text, false, lcd, delay);
        shift_line(false, lcd, delay);

        let mut input = ['\0'; MAX_DISPLAY_CHARS];
        read_line(&mut input, keypad, delay);

        let mut char_buffer = [0; MAX_DISPLAY_CHARS * 4];
        for (i, c) in input.iter().enumerate() {
            // this probably works?? i hope
            // we might get more zeros than i'd hoped
            c.encode_utf8(&mut char_buffer[i * 4..i * 4 + 4]);
        }
        *var = lexical_core::parse::<f64>(&char_buffer).unwrap();
    }

    // presentation screen 1
    let ph_status = calcs::eval_ph(ph);
    let hard_status = calcs::eval_hardness(hard);
    let cond_status = calcs::eval_cond(cond);

    let first_line = "pH " + "2";
    // i'm gonna cry you can actually allocate things

    WaterData {
        ph: ph,
        cond: cond,
        hardness: hard,
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

fn read_line(
    string: &mut [char; MAX_DISPLAY_CHARS],
    keypad: &mut GenericKeypad,
    delay: &mut GenericDelay,
) {
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
    lcd.clear(delay).unwrap();
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

fn shift_line(first_line: bool, lcd: &mut GenericDisplay, delay: &mut GenericDelay) {
    let pos: u8 = if first_line { 0 } else { 40 };
    lcd.set_cursor_pos(pos, delay).unwrap();
}
