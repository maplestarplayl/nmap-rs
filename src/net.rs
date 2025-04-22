use std::{collections::HashMap, fmt::format, sync::{Arc, Mutex}, time::Instant};

use crate::scan::{ self, ScanResult};



pub type Results = HashMap<String, Vec<ScanResult>>;

pub async fn execute_scan(state: Arc<Mutex<Results>>, targets: Vec<String>, ports: String) {
    let mut ips = Vec::new();
    for target in targets {
        let ip = match target.parse::<std::net::IpAddr>() {
            Ok(ip) => vec![ip],
            Err(e) => {
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

    let mut handles = Vec::new();
    for ip in ips {
        let is_alive = ping(ip).await;
        let ports = ports.clone();
        let state = state.clone();
        match is_alive {
            true => {
                let handle = tokio::spawn(async move  {
                    let results = scan::scan_ports(ip, ports.as_str()).await;
                    // for result in results {
                    //     if result.is_open {
                    //         println!("{}:{} is open", ip, result.port);
                    //     } else {
                    //         println!("{}:{} is closed", ip, result.port);
                    //     }
                    // }
                    state.lock().unwrap().insert(ip.to_string(), results);
                    
                });
                handles.push(handle);
            }
            false => {
                state.lock().unwrap().insert(format!("{} is not reachable", ip), Vec::new());
                // println!("{}:{} is not reachable", ip, ports);
            }
        }
    }

    for handle in handles {
        let _ = handle.await;
    }
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