use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use anyhow::{Context, Result};
use ds1307::{DateTimeAccess, Ds1307, NaiveDate};
use esp_idf_hal::i2c;
use esp_idf_hal::i2c::config::MasterConfig;
use esp_idf_hal::prelude::*;
use esp_idf_sys as _;

static NTHREADS: i32 = 3;

fn main() -> Result<()> {
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    println!("Start!");

    // Channels have two endpoints: the `Sender<T>` and the `Receiver<T>`,
    // where `T` is the type of the message to be transferred
    // (type annotation is superfluous)
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    let mut children = Vec::new();

    for id in 0..NTHREADS {
        // The sender endpoint can be copied
        let thread_tx = tx.clone();

        // Each thread will send its id via the channel
        let child = thread::spawn(move || {
            // The thread takes ownership over `thread_tx`
            // Each thread queues a message in the channel
            thread_tx.send(id).unwrap();

            // Sending is a non-blocking operation, the thread will continue
            // immediately after sending its message
            println!("thread {} finished", id);
        });

        children.push(child);
    }

    // Here, all the messages are collected
    let mut ids = Vec::with_capacity(NTHREADS as usize);
    for _ in 0..NTHREADS {
        // The `recv` method picks a message from the channel
        // `recv` will block the current thread if there are no messages available
        ids.push(rx.recv());
    }

    // Wait for the threads to complete any remaining work
    for child in children {
        child.join().expect("oops! the child thread panicked");
    }

    // Show the order in which the messages were sent
    println!("{:?}", ids);

    let peripherals = Peripherals::take().context("Could not initialize peripherals!")?;

    let config = MasterConfig::new().baudrate(400.kHz().into());

    let i2c = i2c::Master::new(
        peripherals.i2c0,
        i2c::MasterPins {
            sda: peripherals.pins.gpio21,
            scl: peripherals.pins.gpio22,
        },
        config,
    )
    .context("Could not initialize I2C!")?;

    let t = thread::spawn(|| {
        let mut rtc = Ds1307::new(i2c);
        let datetime = NaiveDate::from_ymd(2020, 5, 2).and_hms(19, 59, 58);
        rtc.set_datetime(&datetime).unwrap();
        // ...
        let datetime = rtc.datetime().unwrap();
        println!("{}", datetime);
    });

    t.join().map_err(|_| anyhow::Error::msg("Cannot join the thread!"))?;

    Ok(())
}
