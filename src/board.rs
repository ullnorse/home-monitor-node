use crate::error::{AppError, Result};
use esp_hal::{
    Blocking, DriverMode, assign_resources,
    clock::CpuClock,
    i2c::master::{Config as I2cConfig, I2c},
    timer::timg::TimerGroup,
};

assign_resources! {
    Resources<'d> {
        i2c: I2cResources<'d> {
            i2c0: I2C0,
            sda: GPIO21,
            scl: GPIO22,
        },
    }
}

#[derive(Default)]
pub struct Board {
    i2c: Option<I2c<'static, Blocking>>,
}

impl Board {
    pub fn new() -> Result<Self> {
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let peripherals = esp_hal::init(config);

        let r = split_resources!(peripherals);

        let i2c = I2c::new(r.i2c.i2c0, I2cConfig::default())?
            .with_scl(r.i2c.scl)
            .with_sda(r.i2c.sda);

        esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

        let timg0 = TimerGroup::new(peripherals.TIMG0);
        esp_rtos::start(timg0.timer0);

        Ok(Self { i2c: Some(i2c) })
    }

    pub fn take_i2c(&mut self) -> Result<I2c<'static, Blocking>> {
        self.i2c.take().ok_or(AppError::I2cAlreadyTaken)
    }
}
