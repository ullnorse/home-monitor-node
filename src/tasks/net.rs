use defmt::info;
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use esp_radio::wifi::WifiDevice;

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}

#[embassy_executor::task]
pub async fn net_monitor_task(stack: &'static Stack<'static>) {
    stack.wait_config_up().await;
    loop {
        if let Some(cfg) = stack.config_v4() {
            let ip = cfg.address.address();
            info!("Got IPv4: {}", ip);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}
