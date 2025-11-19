use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::i2c::AtomicDevice;
use embedded_hal_bus::util::AtomicCell;
use esp_hal::Blocking;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::I2c;
use esp_hal::timer::timg::TimerGroup;
use ssd1306::Ssd1306;
use static_cell::StaticCell;

use crate::drivers::sht3x::Sht3x;
use crate::drivers::ssd1306::Display;
use crate::error::Result;

use crate::board::Board;
use crate::tasks::{display::display_task, orchestrate::orchestrate_task, sensor::sensor_task};

pub type TempSensor = Sht3x<AtomicDevice<'static, I2c<'static, Blocking>>, Delay>;
pub type BoardDisplay = Display<AtomicDevice<'static, I2c<'static, Blocking>>>;

static I2C_CELL: StaticCell<AtomicCell<I2c<'static, Blocking>>> = StaticCell::new();

pub async fn run(spawner: Spawner) -> Result<()> {
    let mut board = Board::new()?;

    info!("Embassy initialized!");

    //let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    // let (mut _wifi_controller, _interfaces) =
    //     esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
    //         .expect("Failed to initialize Wi-Fi controller");

    let i2c = board.take_i2c()?;
    let i2c_cell = I2C_CELL.init(AtomicCell::new(i2c));

    let sht3x = crate::drivers::sht3x::Sht3x::new(AtomicDevice::new(i2c_cell), Delay::new())?;
    let display = crate::drivers::ssd1306::Display::new(AtomicDevice::new(i2c_cell))?;

    spawner.spawn(orchestrate_task())?;
    spawner.spawn(display_task(display))?;
    spawner.spawn(sensor_task(sht3x))?;

    Ok(())
}
