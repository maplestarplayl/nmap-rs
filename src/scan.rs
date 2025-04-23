use std::{net::{IpAddr, SocketAddr}, time::Duration};

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
        Ok(_) => {
            // println!("{}:{} is open", ip, port);
            ScanResult {
                port,
                is_open: true,
            }
        },
        Err(_) => {
            // println!("{}:{} is closed", ip, port);
            ScanResult {
                port,
                is_open: false,
            }
        }
    }
}

pub async fn scan_ports(ip: IpAddr, ports: &str) -> Vec<ScanResult> {
    let ports_vec = parse_ports_range(ports);
    
    // create multiple concurrent tasks
    let mut handles = Vec::new();
    for port in ports_vec {
        let handle = tokio::spawn(async move {
            scan_port_with_timeout(ip, port, Duration::from_millis(500)).await
        });
        handles.push(handle);
    }
    
    // wait for all port scanning tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }
    
    results
}

pub async fn scan_port_with_timeout(ip: IpAddr, port: u16, timeout: Duration) -> ScanResult {
    let socket_addr = SocketAddr::new(ip, port);
    let socket = TcpSocket::new_v4().unwrap();
    
    // use tokio::time::timeout to limit the connection attempt time
    match tokio::time::timeout(timeout, socket.connect(socket_addr)).await {
        Ok(Ok(_)) => {
            // println!("{}:{} is open", ip, port);
            ScanResult { port, is_open: true }
        },
        _ => {
            // timeout or connection error, consider the port as closed
            // println!("{}:{} is closed", ip, port);
            ScanResult { port, is_open: false }
        }
    }
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