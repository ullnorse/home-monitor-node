use defmt::Format;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;
use sensirion_rht::{Addr, Device, Repeatability, kind};

#[derive(Debug, Format)]
pub enum Sht3xError {
    Bus,
    Timeout,
    InvalidData,
}

#[derive(Debug, Clone, Copy)]
pub struct Sht3xReading {
    pub temperature: f64,
    pub humidity: f64,
}

pub struct Sht3x<I2C, Delay>
where
    I2C: I2c,
    Delay: DelayNs,
{
    inner: Device<I2C, Delay, kind::SHT3x>,
}

impl<I2C, Delay> Sht3x<I2C, Delay>
where
    I2C: I2c,
    Delay: DelayNs,
{
    pub fn new(i2c: I2C, delay: Delay) -> Self {
        let device = Device::new_sht3x(Addr::A, i2c, delay);
        Self { inner: device }
    }

    pub fn read(&mut self) -> Result<Sht3xReading, Sht3xError> {
        self.inner
            .single_shot(Repeatability::High)
            .map(|(t, h)| Sht3xReading {
                temperature: t.as_celsius(),
                humidity: h.as_percent(),
            })
            .map_err(|err| match err {
                sensirion_rht::Error::I2C(_) => Sht3xError::Bus,
                sensirion_rht::Error::Timeout => Sht3xError::Timeout,
                sensirion_rht::Error::CRC => Sht3xError::InvalidData,
            })
    }
}
