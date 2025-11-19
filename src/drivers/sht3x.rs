use embedded_hal::{delay::DelayNs, i2c::I2c};
use sensirion_rht::{Addr, Device, Repeatability, kind};

use crate::error::{AppError, Result};

pub struct Reading {
    pub temperature: f64,
    pub humidity: f64,
}

pub struct Sht3x<I2C, Delay>
where
    I2C: I2c,
    Delay: DelayNs,
{
    device: Device<I2C, Delay, kind::SHT3x>,
}

impl<I2C, Delay, E> Sht3x<I2C, Delay>
where
    I2C: I2c<Error = E>,
    Delay: DelayNs,
{
    pub fn new(i2c: I2C, delay: Delay) -> Result<Self> {
        let device = Device::new_sht3x(Addr::A, i2c, delay);
        Ok(Self { device })
    }

    pub fn single_shot(&mut self) -> Result<Reading> {
        let (temperature, humidity) = self
            .device
            .single_shot(Repeatability::High)
            .map_err(|_| AppError::Sht3x)?;

        Ok(Reading {
            temperature: temperature.as_celsius(),
            humidity: humidity.as_percent(),
        })
    }
}
