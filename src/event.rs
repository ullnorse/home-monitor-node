use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

const EVENT_CHANNEL_SIZE: usize = 10;

pub static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Event, EVENT_CHANNEL_SIZE> =
    Channel::new();

pub async fn send_event(event: Event) {
    EVENT_CHANNEL.send(event).await;
}

pub async fn receive_event() -> Event {
    EVENT_CHANNEL.receive().await
}

#[derive(Debug, Clone)]
pub enum Event {
    SensorValue((f64, f64)),
}
