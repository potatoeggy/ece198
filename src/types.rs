use alloc::{
    fmt::format,
    format,
    string::{String, ToString},
};
use embedded_hal::digital::v2::InputPin;
use hd44780_driver::{bus::FourBitBus, Cursor, CursorBlink, Display, DisplayMode, HD44780};
use keypad2::Keypad;
use libm::sqrt;
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

use self::calcs::{eval_ph, improve_cond, improve_hardness, improve_ph, QualityLevel, Suggestion};
mod calcs;

const MAX_DISPLAY_CHARS: usize = 16;
const PH_STANDARD: f64 = 7.0;
const COND_STANDARD: f64 = 400.0;
const HARD_STANDARD: f64 = 90.0;

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
        Pin<'A', 9, Output>,
        Pin<'A', 8, Output>,
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

pub struct Stat {
    avg: f64,
    stdev: f64,
    standard: f64,
    num_total: usize,
    num_success: usize,
}

impl Stat {
    pub fn new(data: &[f64], standard: f64, qualifier: &dyn Fn(f64) -> QualityLevel) -> Stat {
        Stat {
            avg: calc_avg(data),
            stdev: calc_stdev(data),
            standard: standard,
            num_total: data.len(),
            num_success: data
                .iter()
                .map(|&d| qualifier(d))
                .filter(|&d| d == QualityLevel::Good || d == QualityLevel::Ok)
                .count(),
        }
    }
}

pub fn print_main_menu(lcd: &mut GenericDisplay, delay: &mut GenericDelay) {
    write_screen("1. New data", "2. Summary (x)", lcd, delay);
}

pub fn add_data(
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

        let mut input = [' '; MAX_DISPLAY_CHARS];
        read_line(&mut input, keypad, delay, lcd);

        write_line(input.iter().collect::<String>().trim(), false, lcd, delay);
        *var = input
            .iter()
            .collect::<String>()
            .trim()
            .parse::<f64>()
            .unwrap();
    }

    // presentation screen 1
    let ph_status = calcs::eval_ph(ph);
    let hard_status = calcs::eval_hardness(hard);
    let cond_status = calcs::eval_cond(cond);
    let total_status = if ph_status == QualityLevel::Poor
        || hard_status == QualityLevel::Poor
        || ph_status == QualityLevel::Poor
    {
        QualityLevel::Poor
    } else {
        QualityLevel::Good
    };

    let first_line = format!("pH {}   Cond  {}", ph_status.code(), cond_status.code());
    let second_line = format!("Ha {}   Total {}", hard_status.code(), total_status.code());
    // i'm gonna cry you can actually allocate things
    write_screen(first_line.as_str(), second_line.as_str(), lcd, delay);
    read_char(keypad, delay);

    let improvements: [(
        &str,
        &dyn Fn(Suggestion) -> &'static str,
        &dyn Fn(f64) -> Suggestion,
        &f64,
        &str,
    ); 3] = [
        (
            "pH",
            &|x: Suggestion| match x {
                Suggestion::Add(a) => "add base:",
                Suggestion::Remove(a) => "remove base:",
                Suggestion::None => "Good",
            },
            &improve_ph,
            &ph,
            "mol/L OH-",
        ),
        (
            "Cond",
            &|x: Suggestion| match x {
                Suggestion::Add(a) => "add salt:",
                Suggestion::Remove(a) => "rem. salt:",
                Suggestion::None => "Good",
            },
            &improve_cond,
            &cond,
            "mg/L",
        ),
        (
            "Ha",
            &|x: Suggestion| match x {
                Suggestion::Add(a) => "add CaCO3:",
                Suggestion::Remove(a) => "rem. CaCO3:",
                Suggestion::None => "Good",
            },
            &improve_hardness,
            &hard,
            "mg/L CaCO3",
        ),
    ];

    for (title, desc_gen, improve_gen, &val, units) in improvements {
        let suggestion = improve_gen(val);
        let desc_text = desc_gen(suggestion);
        let first_line = format!("{}: {}", title, desc_text);

        let value = match suggestion {
            Suggestion::None => String::new(),
            Suggestion::Add(x) => x.to_string(),
            Suggestion::Remove(x) => x.to_string(),
        };

        let second_line = format!("{:.2} {}", value, units);

        write_screen(first_line.as_str(), second_line.as_str(), lcd, delay);
        read_char(keypad, delay);
    }

    WaterData {
        ph: ph,
        cond: cond,
        hardness: hard,
    }
}

pub fn summary(data: &[WaterData], keypad: &mut GenericKeypad, delay: &mut GenericDelay) {
    for d in data {
        read_char(keypad, delay);
    }
    // TODO
}

pub fn read_char(keypad: &mut GenericKeypad, delay: &mut GenericDelay) -> char {
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

pub fn read_line(
    string: &mut [char; MAX_DISPLAY_CHARS],
    keypad: &mut GenericKeypad,
    delay: &mut GenericDelay,
    lcd: &mut GenericDisplay,
) {
    // TODO: display text on screen on input
    delay.delay_ms(1_u16);
    let mut index = 0;

    loop {
        let key = keypad.read_char(delay);

        if key != ' ' {
            let mut char = '.';
            if key == '#' && index > 0 {
                // treat as enter
                // do not accept blank input
                break;
            } else if key == '#' {
                continue;
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
                lcd.write_char(char, delay).unwrap();
            }
        }
        delay.delay_ms(100u16);
    }
    for i in index..MAX_DISPLAY_CHARS {
        string[i] = ' ';
    }
}

pub fn write_screen(first: &str, second: &str, lcd: &mut GenericDisplay, delay: &mut GenericDelay) {
    delay.delay_ms(10u16);
    lcd.clear(delay).unwrap();
    lcd.reset(delay).unwrap();
    lcd.write_str(first, delay).unwrap();
    lcd.set_cursor_pos(40u8, delay).unwrap();
    lcd.write_str(second, delay).unwrap();
}

pub fn write_line(
    string: &str,
    second_line: bool,
    lcd: &mut GenericDisplay,
    delay: &mut GenericDelay,
) {
    delay.delay_ms(10u16);

    let pos = if second_line { 40u8 } else { 0u8 };
    lcd.set_cursor_pos(pos, delay).unwrap();
    lcd.write_str(string, delay).unwrap(); // hope it also clears the rest of the line
}

pub fn shift_line(first_line: bool, lcd: &mut GenericDisplay, delay: &mut GenericDelay) {
    let pos: u8 = if first_line { 0 } else { 40 };
    lcd.set_cursor_pos(pos, delay).unwrap();
}

pub fn calc_stdev(data: &[f64]) -> f64 {
    let len = data.len() as f64;
    let mut total = 0.0;
    for &val in data {
        total += val;
    }

    let mean = total / len;
    let mut sum = 0.0;
    for &val in data {
        sum += (val - mean) * (val - mean);
    }

    let variance = sum / len;
    sqrt(variance)
}

pub fn calc_avg(data: &[f64]) -> f64 {
    let len = data.len() as f64;
    let mut total = 0.0;
    for &val in data {
        total += val;
    }
    total / len
}
