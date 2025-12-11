use defmt::info;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embedded_hal_bus::i2c::AtomicDevice;
use embedded_hal_bus::util::AtomicCell;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{Blocking, assign_resources, ram};
use esp_radio::Controller;
use esp_radio::wifi::{AuthMethod, ClientConfig, ModeConfig};
use static_cell::StaticCell;

use crate::drivers::sht3x::Sht3x;
use crate::drivers::ssd1306::Ssd1306;
use crate::error::{AppError, Result};

use crate::tasks::I2cBus;
use crate::tasks::display::display_task;
use crate::tasks::http_client::http_client_task;
use crate::tasks::net::{alive_task, net_task};
use crate::tasks::orchestrate::orchestrate_task;
use crate::tasks::sensor::sensor_task;
use crate::tasks::wifi::wifi_task;

const HOTSPOT_SSID: &str = "Oblakoder-2.4G";
const HOTSPOT_PASSWORD: &str = "skidambundujerjevrelo";

static I2C_CELL: StaticCell<AtomicCell<I2cBus>> = StaticCell::new();
static RADIO_CONTROLLER: StaticCell<Controller> = StaticCell::new();
static NET_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static STACK: StaticCell<Stack> = StaticCell::new();

assign_resources! {
    Resources<'d> {
        i2c: I2cResources<'d> {
            i2c0: I2C0,
            sda: GPIO21,
            scl: GPIO22,
        },
        wifi: WifiResources<'d> {
            wifi: WIFI,
        }
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

    esp_alloc::heap_allocator!(size: 72 * 1024);
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    let radio_controller = RADIO_CONTROLLER.init(esp_radio::init()?);

    let config = ClientConfig::default()
        .with_ssid(HOTSPOT_SSID.into())
        .with_password(HOTSPOT_PASSWORD.into())
        .with_auth_method(AuthMethod::Wpa2Personal);

    let sta_config = ModeConfig::Client(config);

    let (mut wifi_controller, interfaces) =
        esp_radio::wifi::new(radio_controller, resources.wifi.wifi, Default::default())?;

    wifi_controller.set_config(&sta_config).unwrap();

    let device = interfaces.sta;

    let config = embassy_net::Config::dhcpv4(Default::default());

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let (stack, runner) = embassy_net::new(
        device,
        config,
        NET_RESOURCES.init(StackResources::<3>::new()),
        seed,
    );

    let stack = STACK.init(stack);

    let i2c = init_i2c(resources.i2c)?;
    let i2c_cell = I2C_CELL.init(AtomicCell::new(i2c));

    let sht3x = Sht3x::new(AtomicDevice::new(i2c_cell), Delay::new());
    let display = Ssd1306::new(AtomicDevice::new(i2c_cell)).map_err(|_| AppError::Display)?;

    spawner.spawn(orchestrate_task())?;
    spawner.spawn(display_task(display))?;
    spawner.spawn(sensor_task(sht3x))?;
    spawner.spawn(wifi_task(wifi_controller))?;
    spawner.spawn(net_task(runner))?;
    spawner.spawn(http_client_task(stack))?;
    spawner.spawn(alive_task())?;

    Ok(())
}
