#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::string::String;
use tock::adc;
use tock::console::Console;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut console = Console::new();
    let mut with_callback = adc::with_callback(|channel: usize, value: usize| {
        console.write(String::from("channel: "));
        console.write(tock::fmt::u32_as_decimal(channel as u32));
        console.write(String::from(": "));
        console.write(tock::fmt::u32_as_decimal(value as u32));
        console.write(String::from("\n"));
    });

    let adc = with_callback.init().unwrap();

    loop {
        adc.sample(0).unwrap();
        timer::sleep(Duration::from_ms(2000));
    }
}
