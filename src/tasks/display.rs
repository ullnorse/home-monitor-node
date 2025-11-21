use defmt::error;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

use crate::drivers::sht3x::Sht3xReading;
use crate::tasks::DisplayHandle;

pub static DISPLAY_SIGNAL: Signal<CriticalSectionRawMutex, Sht3xReading> = Signal::new();

pub fn update_display_text(data: Sht3xReading) {
    DISPLAY_SIGNAL.signal(data);
}

async fn wait() -> Sht3xReading {
    DISPLAY_SIGNAL.wait().await
}

#[embassy_executor::task]
pub async fn display_task(mut display: DisplayHandle) {
    loop {
        let Sht3xReading {
            temperature,
            humidity,
        } = wait().await;

        if let Err(e) = display.show_sensor_data(temperature, humidity) {
            error!("Display task error: {}", e);
        }
    }
}
