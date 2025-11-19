use defmt::info;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver, signal::Signal,
};

pub static DISPLAY_TEXT: Signal<CriticalSectionRawMutex, (f64, f64)> = Signal::new();

pub fn update_display_text(data: (f64, f64)) {
    DISPLAY_TEXT.signal(data);
}

async fn wait() -> (f64, f64) {
    DISPLAY_TEXT.wait().await
}

#[embassy_executor::task]
pub async fn display_task() {
    loop {
        let (tempareture, humidity) = wait().await;

        info!("Display task - temperature: {}, humidity: {}", tempareture, humidity);
    }
}
