use std::{collections::HashMap, sync::{Arc, Mutex}, time::{Duration, Instant}};

use crate::scan::{ self, ScanResult};



pub type Results = HashMap<String, Vec<ScanResult>>;

pub async fn execute_scan(state: Arc<Mutex<Results>>, targets: Vec<String>, ports: String, time: Arc<Mutex<Duration>>){
    let start_time = Instant::now();

    let mut ips = Vec::new();
    for target in targets {
        let ip = match target.parse::<std::net::IpAddr>() {
            Ok(ip) => vec![ip],
            Err(_) => {
                match parse_cidr(&target) {
                    Ok(ips) => ips,
                    Err(e) => {
                        println!("failed to parse cidr: {}", e);
                        continue;
                    }
                }
            }
        };
        ips.extend(ip);
    }

    let mut ping_handles = Vec::new();
    for ip in ips {
        let ping_future = tokio::spawn(async move {
            let is_alive = ping(ip).await;
            (ip, is_alive)
        });
        ping_handles.push(ping_future);
    }

    let mut ping_results = Vec::new();
    for handle in ping_handles {
        if let Ok(result) = handle.await {
            ping_results.push(result);
        }
    }
    
    let mut scan_handles = Vec::new();
    for (ip, is_alive) in ping_results {
        let ports_clone = ports.clone();
        let state_clone = state.clone();
        
        if is_alive {
            let handle = tokio::spawn(async move {
                let results = scan::scan_ports(ip, ports_clone.as_str()).await;
                // println!("insert ip addr: {}", ip);
                state_clone.lock().unwrap().insert(ip.to_string(), results);
            });
            scan_handles.push(handle);
        } else {
            state.lock().unwrap().insert(format!("{} is not reachable", ip), Vec::new());
        }
    }

    for handle in scan_handles {
        let _ = handle.await;
    }
    
    let end_time = Instant::now();
    *time.lock().unwrap() = end_time.duration_since(start_time);
} 




async fn ping(ip: std::net::IpAddr) -> bool {
    use surge_ping::ping;
    match ping(ip, &[1, 2, 3, 4]).await {
        Ok(_) => {
            true
        }
        Err(_) => {
            false
        }
    }
}

pub fn parse_cidr(cidr: &str) -> Result<Vec<std::net::IpAddr>, Box<dyn std::error::Error>> {
    let network = cidr.parse::<ipnetwork::IpNetwork>()?;
    Ok(network.iter().collect())
}