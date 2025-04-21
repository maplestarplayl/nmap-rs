use std::net::{IpAddr, SocketAddr};

use ratatui::text::Text;
use tokio::net::TcpSocket;
#[derive(Clone)]
pub struct ScanResult {
    pub port: u16,
    pub is_open: bool,
}

impl<'a> Into<Text<'a>> for ScanResult {
    fn into(self) -> Text<'a> {
        Text::from(format!("{} {}", self.port, if self.is_open { "Open" } else { "Closed" }))
    }
}

/// Scan a single port using TCP
pub async fn scan_port(ip: IpAddr, port: u16) -> ScanResult {
    let socket_addr = SocketAddr::new(ip, port);
    let socket = TcpSocket::new_v4().unwrap();
    match socket.connect(socket_addr).await {
        Ok(_) => ScanResult {
            port,
            is_open: true,
        },
        Err(e) => {
            println!("{}", e);
            ScanResult {
                port,
                is_open: false,
            }
        }
    }
}

pub async fn scan_ports(ip: IpAddr, ports: &str) -> Vec<ScanResult> {
    let ports = parse_ports_range(ports);
    let mut results = Vec::new();
    for port in ports {
        results.push(scan_port(ip, port).await);
    }
    results
}

fn parse_ports_range(ports: &str) -> Vec<u16> {
    let mut results = Vec::new();
    if ports.contains('-') {
        // Handle range expressions like "1-100"
        let range_parts: Vec<&str> = ports.split('-').collect();
        if range_parts.len() == 2 {
            if let (Ok(start), Ok(end)) =
                (range_parts[0].parse::<u16>(), range_parts[1].parse::<u16>())
            {
                for port in start..=end {
                    results.push(port);
                }
            }
        }
    } else if let Ok(port) = ports.parse::<u16>() {
        results.push(port);
    }
    results
}