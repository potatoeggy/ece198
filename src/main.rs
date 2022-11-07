#![deny(unsafe_code)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use hal::{
    gpio::{self, Output, PushPull},
    timer::CounterUs,
};
use libm::{pow, powf, sqrt};
// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4::stm32f401::TIM2;
use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*, timer::Channel};

type LedPin = gpio::PA5<Output<PushPull>>;

static G_LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));
static G_TIM: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the LED. On the Nucleo-446RE it"s connected to pin PA5.
        let gpioa = dp.GPIOA.split();
        let gpiod = dp.GPIOD.split();
        let mut led = gpioa.pa5.into_push_pull_output();

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Create a delay abstraction based on SysTick
        let mut delay = cp.SYST.delay(&clocks);

        let buzzer = gpioa.pa9.into_alternate();
        let mut buzz_pwm = dp.TIM1.pwm_hz(buzzer, 2000.Hz(), &clocks);

        let max_duty = buzz_pwm.get_max_duty();
        buzz_pwm.set_duty(Channel::C2, max_duty / 2);

        let tones = [
            ("e0", 165.Hz()),
            ("f0", 175.Hz()),
            ("f0+", 185.Hz()),
            ("g0", 196.Hz()),
            ("g0+", 208.Hz()),
            ("a0", 220.Hz()),
            ("a0+", 233.Hz()),
            ("b0", 245.Hz()),
            ("c", 261.Hz()),
            ("c+", 277.Hz()),
            ("d", 294.Hz()),
            ("d+", 311.Hz()),
            ("e", 329.Hz()),
            ("f", 349.Hz()),
            ("f+", 370.Hz()),
            ("g", 392.Hz()),
            ("g+", 415.Hz()),
            ("a", 440.Hz()),
            ("a+", 466.Hz()),
            ("b", 493.Hz()),
            ("c2", 523.Hz()),
            ("d2", 594.Hz()),
        ];

        let twinkle_twinkle = [
            ("c", 1),
            ("c", 1),
            ("g", 1),
            ("g", 1),
            ("a", 1),
            ("a", 1),
            ("g", 2),
            ("f", 1),
            ("f", 1),
            ("e", 1),
            ("e", 1),
            ("d", 1),
            ("d", 1),
            ("c", 2),
            (" ", 4),
        ];

        let scale = [
            ("c", 1),
            ("c+", 1),
            ("d", 1),
            ("d+", 1),
            ("e", 1),
            ("f", 1),
            ("f+", 1),
            ("g", 1),
            ("g+", 1),
            ("a+", 1),
            ("a", 1),
            ("b", 1),
            (" ", 4),
        ];

        let megalovania = [
            ("d", 1),
            ("d", 1),
            ("d2", 2),
            ("a", 2),
            (" ", 1),
            ("g+", 1),
            (" ", 1),
            ("g", 1),
            (" ", 1),
            ("f", 2),
            ("d", 1),
            ("f", 1),
            ("g", 1),
            // bar 2
            ("c", 1),
            ("c", 1),
            ("d2", 2),
            ("a", 2),
            (" ", 1),
            ("g+", 1),
            (" ", 1),
            ("g", 1),
            (" ", 1),
            ("f", 2),
            ("d", 1),
            ("f", 1),
            ("g", 1),
            // bar 3
            ("b0", 1),
            ("b0", 1),
            ("d2", 2),
            ("a", 2),
            (" ", 1),
            ("g+", 1),
            (" ", 1),
            ("g", 1),
            (" ", 1),
            ("f", 2),
            ("d", 1),
            ("f", 1),
            ("g", 1),
            // bar 3
            ("a0+", 1),
            ("a0+", 1),
            ("d2", 2),
            ("a", 2),
            (" ", 1),
            ("g+", 1),
            (" ", 1),
            ("g", 1),
            (" ", 1),
            ("f", 2),
            ("d", 1),
            ("f", 1),
            ("g", 1),
            // main song
            ("f", 2),
            ("f", 1),
            ("f", 1),
            (" ", 1),
            ("f", 1),
            (" ", 1),
            ("f", 2),
            ("d", 1),
            (" ", 1),
            ("d", 5),
            // bar 2
            ("f", 2),
            ("f", 1),
            ("f", 1),
            (" ", 1),
            ("g", 1),
            (" ", 1),
            ("g+", 3),
            ("g", 2),
            ("d", 1),
            ("f", 1),
            ("g", 1),
            (" ", 10000),
        ];

        let mario = [
            ("e", 2),
            ("e", 2),
            (" ", 2),
            ("e", 2),
            (" ", 2),
            ("c", 2),
            ("e", 4),
            ("g", 4),
            (" ", 4),
            ("g0", 4),
            (" ", 4),
            // main part
            ("c", 4),
            (" ", 2),
            ("g0", 4),
            (" ", 2),
            ("e0", 4),
            (" ", 2),
            ("a0", 4),
            ("b0", 4),
            ("a0+", 2),
            ("a0", 4),
            ("g0", 3),
            ("e", 3),
            ("g", 3),
            ("a", 4),
            ("f", 2),
            ("g", 2),
            (" ", 2),
            ("e", 4),
            ("c", 2),
            ("d", 2),
            ("b0", 4),
            (" ", 10000),
        ];

        let tune = mario;

        let tempo = 60_u32;

        loop {
            // 1. Obtain a note in the tune
            for note in tune {
                // 2. Retrieve the freqeuncy and beat associated with the note
                for tone in tones {
                    // 2.1 Find a note match in the tones array and update frequency and beat variables accordingly
                    if tone.0 == note.0 {
                        // 3. Play the note for the desired duration (beats*tempo)
                        // 3.1 Adjust period of the PWM output to match the new frequency
                        buzz_pwm.set_period(tone.1);
                        // 3.2 Enable the channel to generate desired PWM
                        buzz_pwm.enable(Channel::C2);
                        // 3.3 Keep the output on for as long as required
                        delay.delay_ms(note.1 * tempo);
                    } else if note.0 == " " {
                        // 2.2 if " " tone is found disable output for one beat
                        buzz_pwm.disable(Channel::C2);
                        delay.delay_ms(note.1 * tempo / (tempo));
                    }
                }
                // 4. Silence for half a beat between notes
                // 4.1 Disable the PWM output (silence)
                buzz_pwm.disable(Channel::C2);
                // 4.2 Keep the output off for half a beat between notes
                delay.delay_ms(tempo / 2);
                // 5. Go back to 1.
            }
        }
    }

    loop {}
}

fn calc_mean(data: &[f64]) -> f64 {
    let mut total = 0.0;
    for &val in data {
        total += val;
    }
    total / (data.len() as f64)
}

fn calc_stdev(data: &[f64]) -> f64 {
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

fn calc_median(data: &mut [f64]) -> f64 {
    bubble_sort(data);

    // TODO: finish
    0.0
}

fn bubble_sort(data: &mut [f64]) {
    // no builtin sorting in rust without std
    let mut new_len: usize;
    let mut len = data.len();

    loop {
        new_len = 0;
        for i in 1..len {
            if data[i - 1] > data[i] {
                data.swap(i - 1, i);
                new_len = i;
            }
        }

        if new_len == 0 {
            break;
        }

        len = new_len;
    }
}

fn ph_to_hconc(ph: f64) -> f64 {
    pow(10.0, -ph)
}
