use defmt::info;
use embassy_executor::Spawner;
use embedded_hal_bus::i2c::AtomicDevice;
use embedded_hal_bus::util::AtomicCell;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{Blocking, assign_resources};
use static_cell::StaticCell;

use crate::drivers::sht3x::Sht3x;
use crate::drivers::ssd1306::Ssd1306;
use crate::error::{AppError, Result};

use crate::tasks::I2cBus;
use crate::tasks::display::display_task;
use crate::tasks::orchestrate::orchestrate_task;
use crate::tasks::sensor::sensor_task;

static I2C_CELL: StaticCell<AtomicCell<I2cBus>> = StaticCell::new();

assign_resources! {
    Resources<'d> {
        i2c: I2cResources<'d> {
            i2c0: I2C0,
            sda: GPIO21,
            scl: GPIO22,
        },
    }
}

fn init_i2c<'d>(r: I2cResources<'d>) -> Result<I2c<'d, Blocking>> {
    let i2c = I2c::new(r.i2c0, I2cConfig::default())?
        .with_scl(r.scl)
        .with_sda(r.sda);

    Ok(i2c)
}

pub async fn run(spawner: Spawner) -> Result<()> {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let resources = split_resources!(peripherals);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    let i2c = init_i2c(resources.i2c)?;
    let i2c_cell = I2C_CELL.init(AtomicCell::new(i2c));

    let sht3x = Sht3x::new(AtomicDevice::new(i2c_cell), Delay::new());
    let display = Ssd1306::new(AtomicDevice::new(i2c_cell)).map_err(|_| AppError::Display)?;

    spawner.spawn(orchestrate_task())?;
    spawner.spawn(display_task(display))?;
    spawner.spawn(sensor_task(sht3x))?;

    Ok(())
}
