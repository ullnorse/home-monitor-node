use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::drivers::sht3x::Sht3xReading;
use crate::tasks::wifi::WifiState;

const EVENT_CHANNEL_SIZE: usize = 10;

static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Event, EVENT_CHANNEL_SIZE> = Channel::new();

pub async fn send_event(event: Event) {
    EVENT_CHANNEL.send(event).await;
}

pub async fn receive_event() -> Event {
    EVENT_CHANNEL.receive().await
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    SensorReading(Sht3xReading),
    WifiStatus(WifiState),
}
