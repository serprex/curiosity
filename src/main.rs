extern crate docker;
extern crate cosmos;
extern crate rustc_serialize;
extern crate time;

mod container;

use std::{env, thread};
use std::sync::mpsc::{self, Sender, Receiver};
use cosmos::Cosmos;

fn get_containers(tx: &Sender<String>) {
    let tx = tx.clone();
    loop {
        let containers = match container::get_containers_as_str() {
            Ok(containers) => containers,
            Err(e) => { println!("{}", e); continue; }
        };

        match tx.send(containers) {
            Ok(_) => {}
            Err(e) => { println!("{}", e); continue; }
        };
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

fn run(host: &str, planet_name: &str, interval: u64) {
    let host = host.to_string();
    let planet_name = planet_name.to_string();
    
    let mut last_timestamp = time::precise_time_s() as u64;
    let mut last_containers = String::new();
    
    let (container_tx, container_rx) = mpsc::channel();
    let (cosmos_tx, cosmos_rx) = mpsc::channel();
    
    thread::spawn(move|| { get_containers(&container_tx); });
    thread::spawn(move|| { post_containers(&cosmos_rx, &host, &planet_name); });

    loop {
        let current_timestamp = time::precise_time_s() as u64;
        let diff = current_timestamp - last_timestamp;
        
        if diff >= interval {
            let containers = last_containers.clone();
            match cosmos_tx.send(containers) {
                Ok(_) => {}
                Err(e) => { println!("{}", e); continue; }
            };
            last_timestamp = current_timestamp;
        }
        
        let containers = match container_rx.try_recv() {
            Ok(containers) => containers,
            Err(_) => { thread::sleep_ms(100); continue; }
        };
        
        last_containers = containers.to_string();
    }
}

fn main() {
    let host = match env::var("COSMOS_HOST") {
        Ok(host) => host,
        Err(_) => { panic!("COSMOS_HOST envrionment variable does not exist.") }
    };

    let planet_name = match env::var("COSMOS_PLANET_NAME") {
        Ok(planet_name) => planet_name,
        Err(_) => {
            let planet_name = match container::get_hostname() {
                Ok(planet_name) => planet_name,
                Err(e) => { panic!("{}", e); }
            };
            planet_name
        }
    };

    let interval: u64 = 10;

    run(&host, &planet_name, interval);
}
