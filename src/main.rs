#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::error;
use embassy_executor::Spawner;
use home_monitor_node::app;
use {esp_backtrace as _, esp_println as _};
extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    if let Err(e) = app::run(spawner).await {
        error!("Error during app::run - {}", e);
    }
}
