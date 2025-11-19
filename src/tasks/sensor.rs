use defmt::info;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Sender};
use embassy_time::{Duration, Timer};

use crate::{app::TempSensor, drivers::sht3x::Reading, event::{Event, send_event}};


#[embassy_executor::task]
pub async fn sensor_task(mut sensor: TempSensor) {
    loop {
        info!("Sensor task - sending value");

        if let Ok(Reading {temperature, humidity}) = sensor.single_shot() {
            send_event(Event::SensorValue((temperature, humidity))).await;
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}
