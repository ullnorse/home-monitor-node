use core::net::Ipv4Addr;

use defmt::{error, info, warn};
use embassy_net::tcp::{TcpSocket, State}; // Import State enum
use embassy_net::{IpEndpoint, Stack};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use reqwless::client::HttpClient;

use crate::drivers::sht3x::Sht3xReading;

static HTTP_DATA_SIGNAL: Signal<CriticalSectionRawMutex, Sht3xReading> = Signal::new();

pub fn send_sensor_data(data: Sht3xReading) {
    HTTP_DATA_SIGNAL.signal(data);
}

async fn wait_for_reading() -> Sht3xReading {
    HTTP_DATA_SIGNAL.wait().await
}

const SERVER_IP: [u8; 4] = [192, 168, 100, 14];
const SERVER_PORT: u16 = 8080;

fn server_endpoint() -> IpEndpoint {
    let ip = Ipv4Addr::new(SERVER_IP[0], SERVER_IP[1], SERVER_IP[2], SERVER_IP[3]);
    IpEndpoint::new(ip.into(), SERVER_PORT)
}

#[embassy_executor::task]
pub async fn http_client_task(stack: &'static Stack<'static>) {
    info!("http_client: task start");

    stack.wait_config_up().await;
    if let Some(cfg) = stack.config_v4() {
        info!("http_client: network up, my IP = {}", cfg.address);
    }

    let mut rx_buf = [0u8; 1024];
    let mut tx_buf = [0u8; 1024];
    
    let mut socket = TcpSocket::new(*stack, &mut rx_buf, &mut tx_buf);
    socket.set_timeout(Some(Duration::from_secs(20)));
    
    // NEW: Enable TCP Keep-Alive (Heartbeat every 15 seconds)
    // This helps the socket detect if the server died silently.
    socket.set_keep_alive(Some(Duration::from_secs(15)));

    let remote = server_endpoint();

    loop {
        info!("http_client: waiting for reading");
        let reading = wait_for_reading().await;

        // 1. CHECK STATE: The "Source of Truth"
        if socket.state() != State::Established {
            info!("http_client: socket not open (state: {:?}), connecting...", socket.state());
            
            // If it's in a weird limbo state (like TimeWait or CloseWait), forcefully reset it.
            if socket.state() != State::Closed {
                warn!("http_client: forcing cleanup of old state");
                socket.abort();
            }

            match socket.connect(remote).await {
                Ok(()) => info!("http_client: connected"),
                Err(e) => {
                    warn!("http_client: connect error: {:?}", e);
                    Timer::after(Duration::from_secs(3)).await;
                    continue; // Retry connection next loop
                }
            }
        }

        let mut json_buf = [0u8; 100];
        let json_len = serde_json_core::to_slice(&reading, &mut json_buf).unwrap();

        let mut req_buf = [0u8; 256];
        let request = format_no_std::show(
            &mut req_buf,
            format_args!(
                "POST /reading HTTP/1.1\r\n\
Host: local\r\n\
Content-Type: application/json\r\n\
Content-Length: {}\r\n\
Connection: keep-alive\r\n\
\r\n",
                json_len
            ),
        ).unwrap();

        // ---- Send Request ----
        if let Err(e) = socket.write(request.as_bytes()).await {
            warn!("http_client: write header error: {:?}, aborting", e);
            socket.abort();
            continue;
        }

        if let Err(e) = socket.write(&json_buf).await {
            warn!("http_client: write body error: {:?}, aborting", e);
            socket.abort();
            continue;
        }

        info!("http_client: before flush");

        if let Err(e) = socket.flush().await {
            warn!("http_client: flush error: {:?}, aborting", e);
            socket.abort();
            continue;
        }

        info!("http_client: after flush");

        // ---- Read Response ----
        let mut resp_buf = [0u8; 1024];
        
        match embassy_time::with_timeout(Duration::from_millis(500), socket.read(&mut resp_buf)).await {
            Ok(Ok(0)) => {
                warn!("http_client: server closed connection (EOF)");
                socket.abort(); 
            }
            Ok(Ok(n)) => {
                info!("http_client: OK, received {} bytes", n);
                // Socket remains Established for next loop!
            }
            Ok(Err(e)) => {
                warn!("http_client: read error: {:?}, aborting", e);
                socket.abort();
            }
            Err(_) => {
                // Timeout implies server is keeping connection open but sent no extra data.
                // This is the expected "Happy Path" for Keep-Alive.
                info!("http_client: transaction complete (connection kept alive)");
            }
        }

        info!("http_client: end of loop");
    }
}