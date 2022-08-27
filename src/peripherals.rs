use anyhow::Context;
use esp_idf_hal::i2c;
use esp_idf_hal::i2c::config::MasterConfig;
use esp_idf_hal::i2c::Master;
use esp_idf_hal::prelude::{FromValueType, Peripherals};
use shared_bus::{BusManagerStd, I2cProxy};

trait RWPin: esp_idf_hal::gpio::OutputPin + esp_idf_hal::gpio::InputPin {}

pub type I2CBUSInner = esp_idf_hal::i2c::Master<
    esp_idf_hal::i2c::I2C0,
    esp_idf_hal::gpio::Gpio21<esp_idf_hal::gpio::Unknown>,
    esp_idf_hal::gpio::Gpio22<esp_idf_hal::gpio::Unknown>,
>;

pub type I2CBUS<'a> = I2cProxy<'a, std::sync::Mutex<I2CBUSInner>>;

#[derive(Clone)]
pub struct SimplePeripherals {
    inner_i2c: &'static BusManagerStd<I2CBUSInner>,
}

impl TryFrom<Peripherals> for SimplePeripherals {
    type Error = anyhow::Error;

    fn try_from(p: Peripherals) -> Result<Self, Self::Error> {
        let config = MasterConfig::new().baudrate(400.kHz().into());

        let master = Master::new(
            p.i2c0,
            i2c::MasterPins {
                sda: p.pins.gpio21,
                scl: p.pins.gpio22,
            },
            config,
        )
        .context("Could not initialize I2C!")?;

        Ok(SimplePeripherals {
            inner_i2c: shared_bus::new_std!(I2CBUSInner = master).ok_or_else(|| anyhow::Error::msg("Could not create shared I2C bus!"))?,
        })
    }
}

impl SimplePeripherals {
    pub fn i2c(&self) -> I2CBUS {
        self.inner_i2c.acquire_i2c()
    }
}

// #[derive(Clone)]
// pub struct I2C<SDA, SCL>(Arc<Mutex<Master<I2C0, SDA, SCL>>>)
// where
//     SDA: OutputPin + InputPin + ?Sized,
//     SCL: OutputPin + InputPin + ?Sized;
//
// impl<SDA, SCL> I2C<SDA, SCL>
// where
//     SDA: OutputPin + InputPin,
//     SCL: OutputPin + InputPin,
// {
//     fn try_from(inner: I2C0, sda: SDA, scl: SCL) -> Result<Self> {
//         let config = MasterConfig::new().baudrate(400.kHz().into());
//
//         let master = Master::new(inner, i2c::MasterPins { sda, scl }, config).context("Could not initialize I2C!")?;
//
//         Ok(I2C(Arc::new(Mutex::new(master))))
//     }
// }
//
// impl<SDA, SCL> embedded_hal::blocking::i2c::Read for I2C<SDA, SCL>
// where
//     SDA: OutputPin + InputPin,
//     SCL: OutputPin + InputPin,
// {
//     type Error = I2cError;
//
//     fn read(&mut self, address: SevenBitAddress, buffer: &mut [u8]) -> Result<(), Self::Error> {
//         self.0.lock().expect("Mutex was poisoned!").read(address, buffer)
//     }
// }
//
// impl<SDA, SCL> embedded_hal::blocking::i2c::Write for I2C<SDA, SCL>
// where
//     SDA: OutputPin + InputPin,
//     SCL: OutputPin + InputPin,
// {
//     type Error = I2cError;
//
//     fn write(&mut self, address: SevenBitAddress, bytes: &[u8]) -> std::result::Result<(), Self::Error> {
//         self.0.lock().expect("Mutex was poisoned!").write(address, bytes)
//     }
// }
//
// impl<SDA, SCL> embedded_hal::blocking::i2c::WriteRead for I2C<SDA, SCL>
// where
//     SDA: OutputPin + InputPin,
//     SCL: OutputPin + InputPin,
// {
//     type Error = I2cError;
//
//     fn write_read(&mut self, address: SevenBitAddress, bytes: &[u8], buffer: &mut [u8]) -> std::result::Result<(), Self::Error> {
//         self.0.lock().expect("Mutex was poisoned!").write_read(address, bytes, buffer)
//     }
// }
