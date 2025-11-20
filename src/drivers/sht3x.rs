use embedded_hal::{delay::DelayNs, i2c::I2c};
use sensirion_rht::{Addr, Device, Repeatability, kind};

use crate::core::environment_sensor::{
    EnvironmentReading, EnvironmentSensor, EnvironmentSensorError,
};

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

impl<I2C, Delay> Sht3x<I2C, Delay>
where
    I2C: I2c,
    Delay: DelayNs,
{
    pub fn new(i2c: I2C, delay: Delay) -> Self {
        let device = Device::new_sht3x(Addr::A, i2c, delay);
        Self { device }
    }
}

impl<I2C, Delay> EnvironmentSensor for Sht3x<I2C, Delay>
where
    I2C: I2c,
    Delay: DelayNs,
{
    fn read(&mut self) -> Result<EnvironmentReading, EnvironmentSensorError> {
        self.device
            .single_shot(Repeatability::High)
            .map(|(t, h)| EnvironmentReading {
                temperature_c: t.as_celsius(),
                humidity_rh: h.as_percent(),
            })
            .map_err(|err| match err {
                sensirion_rht::Error::I2C(_) => EnvironmentSensorError::Bus,
                sensirion_rht::Error::Timeout => EnvironmentSensorError::Timeout,
                sensirion_rht::Error::CRC => EnvironmentSensorError::InvalidData,
            })
    }
}
