use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use time;
use cosmos::Cosmos;
use container;
use container::Container;
use rustc_serialize::json;

pub struct Curiosity;

impl Curiosity {
    pub fn new() -> Curiosity {
        return Curiosity;
    }

    pub fn run(&self, host: &str, planet_name: &str, interval: u64) {
        let host = host.to_string();
        let planet_name = planet_name.to_string();
        
        let mut timestamp = time::precise_time_s() as u64;
        let mut map: HashMap<String, Container> = HashMap::new();

        let (container_tx, container_rx) = mpsc::channel();
        let (cosmos_tx, cosmos_rx) = mpsc::channel();
        thread::spawn(move|| { Curiosity::get_containers(&container_tx); });
        thread::spawn(move|| { Curiosity::post_containers(&cosmos_rx, &host, &planet_name); });

        loop {
            let containers = match container_rx.try_recv() {
                Ok(containers) => containers,
                Err(_) => { thread::sleep_ms(100); continue; }
            };

            for container in containers.iter() {
                let value = match map.entry(container.Id.clone()) {
                    Occupied(mut entry) => { entry.get_mut().clone() }
                    Vacant(entry) => { entry.insert(container.clone()); continue; }
                };
                let max = value.Stats.Cpu.TotalUtilization;
                let current = container.Stats.Cpu.TotalUtilization;
                if current - max > 0.0 {
                    map.remove(&container.Id);
                    map.insert(container.Id.clone(), container.clone());
                }
            }

            let now = time::precise_time_s() as u64;
            let diff = now - timestamp;
            if diff < interval { continue; }
            timestamp = now;

            let mut cosmos_containers: Vec<Container> = Vec::new();
            for x in map.values() { cosmos_containers.push(x.clone()); }

            let body = match json::encode(&cosmos_containers) {
                Ok(encoded) => encoded,
                Err(e) => { println!("{}", e); continue; }
            };

            match cosmos_tx.send(body) {
                Ok(_) => {}
                Err(e) => { println!("{}", e); continue; }
            }

            map.clear();
        }
    }
    
    fn get_containers(tx: &Sender<Vec<Container>>) {
        let tx = tx.clone();
        loop {
            let containers = match container::get_containers() {
                Ok(containers) => containers,
                Err(e) => { println!("{}", e); continue; }
            };

            let (stats_tx, stats_rx) = mpsc::channel();
            let mut index: u64 = 0;
            for container in containers.iter() {
                if container.Status.contains("Up") == false { continue; }

                let container = container.clone();
                let stats_tx = stats_tx.clone();
                thread::spawn(move|| {
                    let result = container::get_stats_as_cosmos_container(&container);
                    match stats_tx.send(result) {
                        Ok(_) => {}
                        Err(e) => { println!("{}", e); return; }
                    }
                });
                index += 1;
            }

            let mut cosmos_containers: Vec<Container> = Vec::new();
            for _ in 0..index {
                let result = match stats_rx.recv() {
                    Ok(result) => result,
                    Err(e) => { println!("{}", e); continue; }
                };

                let cosmos_container = match result {
                    Ok(container) => container,
                    Err(e) => { println!("{}", e); continue; }
                };

                cosmos_containers.push(cosmos_container);
            }

            match tx.send(cosmos_containers) {
                Ok(_) => {}
                Err(e) => { println!("{}", e); continue; }
            }
        }
    }

    fn post_containers(rx: &Receiver<String>, host: &str, planet_name: &str) {
        let rx = rx.clone();
        loop {
            let containers = match rx.recv() {
                Ok(containers) => containers,
                Err(_) => { continue; }
            };

            let cosmos = Cosmos::new(host);
            match cosmos.post_containers(planet_name, &containers) {
                Ok(_) => {}
                Err(e) => { println!("{}", e); }
            };
        }
    }
}
