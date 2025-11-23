use embedded_hal_bus::i2c::AtomicDevice;
use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::I2c;

use crate::drivers::sht3x::Sht3x;
use crate::drivers::ssd1306::Ssd1306;

pub mod display;
pub mod http_client;
pub mod net;
pub mod orchestrate;
pub mod sensor;
pub mod wifi;

pub type I2cBus = I2c<'static, Blocking>;

pub type SensorHandle = Sht3x<AtomicDevice<'static, I2cBus>, Delay>;
pub type DisplayHandle = Ssd1306<AtomicDevice<'static, I2cBus>>;
