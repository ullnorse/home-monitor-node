use core::net::Ipv4Addr;

use defmt::{error, info, warn};
use embassy_net::tcp::TcpSocket;
use embassy_net::{IpEndpoint, Stack};
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
pub async fn http_client_task(stack: &'static Stack<'static>) {
    // Wait for DHCP
    stack.wait_config_up().await;
    if let Some(cfg) = stack.config_v4() {
        info!("http_client: network up, my IP = {}", cfg.address);
    }

    // TODO: change to your server IP + port
    let server_ip = Ipv4Addr::new(172, 20, 10, 9); // laptop / local server
    let server_port = 8090u16;

    loop {
        info!("http_client: connecting to {}:{}", server_ip, server_port);

        let mut rx_buf = [0u8; 2048];
        let mut tx_buf = [0u8; 512];
        let mut socket = TcpSocket::new(*stack, &mut rx_buf, &mut tx_buf);
        socket.set_timeout(Some(Duration::from_secs(10)));

        let remote = IpEndpoint::new(server_ip.into(), server_port);

        if let Err(e) = socket.connect(remote).await {
            warn!("http_client: connect error: {:?}", e);
            Timer::after(Duration::from_secs(5)).await;
            continue;
        }

        info!("http_client: connected, sending POST /readings");

        // ---- build JSON body ----
        // For now, hard-coded example. Later you plug in real sensor values.
        let json_body = br#"{"temperature":23.5,"humidity":45.2}"#;

        // Content-Length header must be body length in bytes
        let mut len_buf = [0u8; 8];
        let content_length =
            format_no_std::show(&mut len_buf, format_args!("{}", json_body.len())).unwrap();

        // ---- build HTTP request ----

        // Minimal HTTP/1.1 POST
        // Host can be anything; many local servers don’t care.
        let mut req = [0u8; 512];

        let s = format_no_std::show(
            &mut req[..],
            format_args!(
                "POST /readings HTTP/1.1\r\n\
                 Host: local\r\n\
                 Content-Type: application/json\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\
                 \r\n",
                content_length
            ),
        ).unwrap();

        // send headers
        if let Err(e) = socket.write(s.as_bytes()) .await {
            error!("http_client: write headers error: {:?}", e);
            socket.close();
            Timer::after(Duration::from_secs(5)).await;
            continue;
        }

        // send body
        if let Err(e) = socket.write(json_body).await {
            error!("http_client: write body error: {:?}", e);
            socket.close();
            Timer::after(Duration::from_secs(5)).await;
            continue;
        }

        if let Err(e) = socket.flush().await {
            error!("http_client: flush error: {:?}", e);
        }

        info!("http_client: POST sent, closing");
        socket.close();

        // send every 10s (adjust as you like)
        Timer::after(Duration::from_secs(10)).await;
    }
}
