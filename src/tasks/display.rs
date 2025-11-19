use defmt::info;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver, signal::Signal,
};

use crate::app::BoardDisplay;

pub static DISPLAY_TEXT: Signal<CriticalSectionRawMutex, (f64, f64)> = Signal::new();

pub fn update_display_text(data: (f64, f64)) {
    DISPLAY_TEXT.signal(data);
}

async fn wait() -> (f64, f64) {
    DISPLAY_TEXT.wait().await
}

#[embassy_executor::task]
pub async fn display_task(mut display: BoardDisplay) {
    let mut buf = [0u8; 128];

    loop {
        let (tempareture, humidity) = wait().await;

        let s = format_no_std::show(&mut buf, format_args!("temp: {}, humidity: {}", tempareture, humidity)).unwrap();

        display.show_text(s);

        info!("Display task - temperature: {}, humidity: {}", tempareture, humidity);
    }
}
