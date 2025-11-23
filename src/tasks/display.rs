use defmt::{Format, error};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

use crate::tasks::DisplayHandle;
use crate::tasks::wifi::WifiState;

#[derive(Debug, Clone, Copy, Format)]
pub struct DisplayData {
    pub temperature: f64,
    pub humidity: f64,
    pub wifi_state: WifiState,
}

impl DisplayData {
    pub fn new(temperature: f64, humidity: f64, wifi_state: WifiState) -> Self {
        DisplayData {
            temperature,
            humidity,
            wifi_state,
        }
    }
}

static DISPLAY_SIGNAL: Signal<CriticalSectionRawMutex, DisplayData> = Signal::new();

pub fn update_display_text(data: DisplayData) {
    DISPLAY_SIGNAL.signal(data);
}

async fn wait() -> DisplayData {
    DISPLAY_SIGNAL.wait().await
}

#[embassy_executor::task]
pub async fn display_task(mut display: DisplayHandle) {
    loop {
        let data = wait().await;

        if let Err(e) =
            display.show_sensor_data(data.temperature, data.humidity, data.wifi_state.into())
        {
            error!("Display task error: {}", e);
        }
    }
}
