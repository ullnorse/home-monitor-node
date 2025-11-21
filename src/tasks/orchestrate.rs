use crate::events::{Event, receive_event};
use crate::tasks::display::update_display_text;

#[embassy_executor::task]
pub async fn orchestrate_task() {
    loop {
        let event = receive_event().await;

        process_event(event).await;
    }
}

async fn process_event(event: Event) {
    match event {
        Event::SensorReading(data) => {
            update_display_text(data);
        }
    }
}
