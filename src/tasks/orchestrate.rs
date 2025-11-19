use defmt::info;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Sender};
use embassy_time::{Duration, Timer};

use crate::{
    event::{Event, receive_event},
    tasks::display::update_display_text,
};

#[embassy_executor::task]
pub async fn orchestrate_task() {
    loop {
        let event = receive_event().await;

        process_event(event).await;
    }
}

async fn process_event(event: Event) {
    match event {
        Event::SensorValue(data) => {
            info!("Received sensor value");
            update_display_text(data);
        }
    }
}
