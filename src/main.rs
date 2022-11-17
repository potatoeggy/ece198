#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

extern crate alloc;
extern crate panic_halt;
use alloc_cortex_m::CortexMHeap;

mod types;
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
use types::{
    add_data, print_main_menu, read_char, GenericDelay, GenericDisplay, GenericKeypad, WaterData,
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
// D6:  D8 / PA9
// D7:  D7 / PA8
// BLA:   5V
// BLK:   GND

// Keypad connections:
// from left to right:
// D0 / PA3 (C2)
// D1 / PA2 (R1)
// D2 / PA10 (C1)
// D3 / PB3 (R4)
// D4 / PB5 (C3)
// D5 / PB4 (R3)
// D6 / PB10 (R2)

// max chars in display

///static data: [WaterData] = [WaterData; 10];

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
    }

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
    let d6 = gpioa.pa9.into_push_pull_output();
    let d7 = gpioa.pa8.into_push_pull_output();

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

    let data_points: [WaterData; 5] = [
        WaterData::new(),
        WaterData::new(),
        WaterData::new(),
        WaterData::new(),
        WaterData::new(),
    ];

    #[allow(clippy::empty_loop)]
    loop {
        print_main_menu(&mut lcd, &mut delay);
        let c = read_char(&mut keypad, &mut delay);
        if c == '*' || c == '#' {
            continue;
        }
        let c_int = c.to_digit(10).unwrap();
        add_data(&mut keypad, &mut lcd, &mut delay);
    }
}
