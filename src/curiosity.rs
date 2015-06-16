use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender, Receiver};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use cosmos::{self, Cosmos};
use docker;
use container;
use time;

pub struct Curiosity;

impl Curiosity {
    pub fn new() -> Curiosity {
        return Curiosity;
    }

    pub fn run(&self, host: &str, interval: u64) {
        let docker = Arc::new(match container::get_docker() {
            Ok(docker) => docker,
            Err(_) => {
                println!("A Docker host was not found.");
                return;
            }
        });

        let host = host.to_string();
        let planet = match container::get_hostname(&docker.clone()) {
            Ok(hostname) => hostname,
            Err(_) => "unnamed".to_string()
        };

        println!("{} is used for the planet.", planet);
        
        let mut timestamp = time::precise_time_s() as u64;
        let mut map: HashMap<String, cosmos::Container> = HashMap::new();
        
        let (container_tx, container_rx) = mpsc::channel();
        let (cosmos_tx, cosmos_rx) = mpsc::channel();
        
        thread::spawn(move|| { Curiosity::get_containers(&docker.clone(), &container_tx); });
        thread::spawn(move|| { Curiosity::post_metrics(&cosmos_rx, &host, &planet); });

        loop {
            let containers = match container_rx.try_recv() {
                Ok(containers) => containers,
                Err(_) => { thread::sleep_ms(100); continue; }
            };

            for container in containers.iter() {
                let value = match map.entry(container.Container.clone()) {
                    Occupied(mut entry) => { entry.get_mut().clone() }
                    Vacant(entry) => { entry.insert(container.clone()); continue; }
                };
                let max = value.Cpu;
                let current = container.Cpu;
                if current - max > 0.0 {
                    map.remove(&container.Container);
                    map.insert(container.Container.clone(), container.clone());
                }
            }

            let now = time::precise_time_s() as u64;
            let diff = now - timestamp;
            if diff < interval { continue; }
            timestamp = now;

            let mut cosmos_containers: Vec<cosmos::Container> = Vec::new();
            for x in map.values() { cosmos_containers.push(x.clone()); }

            match cosmos_tx.send(cosmos_containers) {
                Ok(_) => {}
                Err(e) => { println!("{}", e); continue; }
            }

            map.clear();
        }
    }
    
    fn get_containers(docker: &Arc<docker::Docker>, tx: &Sender<Vec<cosmos::Container>>) {
        let tx = tx.clone();
        
        loop {
            let docker = docker.clone();
            let containers = match container::get_containers(&*docker) {
                Ok(containers) => containers,
                Err(e) => { println!("{}", e); continue; }
            };

            let (stats_tx, stats_rx) = mpsc::channel();
            let mut index: u64 = 0;
            for container in containers.iter() {
                if container.Status.contains("Up") == false { continue; }

                let container = container.clone();
                let stats_tx = stats_tx.clone();
                let docker = docker.clone();
                
                thread::spawn(move|| {
                    let result = container::get_stats_as_cosmos_container(&*docker, &container);
                    match stats_tx.send(result) {
                        Ok(_) => {}
                        Err(e) => { println!("{}", e); return; }
                    }
                });
                index += 1;
            }

            let mut cosmos_containers: Vec<cosmos::Container> = Vec::new();
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

    fn post_metrics(rx: &Receiver<Vec<cosmos::Container>>, host: &str, planet: &str) {
        let rx = rx.clone();
        loop {
            let containers = match rx.recv() {
                Ok(containers) => containers,
                Err(_) => { continue; }
            };

            let cosmos = Cosmos::new(host, planet);
            let res = match cosmos.post_metrics(&containers) {
                Ok(res) => res,
                Err(_) => { println!("Any response was not received."); continue; }
            };

            println!("{}\n{}", res.status_code, res.body);
        }
    }
}
