use defmt::error;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use crate::{
    app::BoardDisplay,
    core::{display::Display, environment_sensor::EnvironmentReading},
};

pub static DISPLAY_TEXT: Signal<CriticalSectionRawMutex, EnvironmentReading> = Signal::new();

pub fn update_display_text(data: EnvironmentReading) {
    DISPLAY_TEXT.signal(data);
}

async fn wait() -> EnvironmentReading {
    DISPLAY_TEXT.wait().await
}

#[embassy_executor::task]
pub async fn display_task(display: BoardDisplay) {
    display_loop(display).await
}

async fn display_loop(mut display: impl Display) {
    loop {
        let reading = wait().await;

        if let Err(e) = display.show_environment(reading) {
            error!("Display task error: {}", e);
        }
    }
}
