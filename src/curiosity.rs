use std::thread;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, Receiver};
use time;
use cosmos::Cosmos;
use container;

pub struct Curiosity;

impl Curiosity {
    pub fn new() -> Curiosity {
        return Curiosity;
    }

    pub fn run(&self, host: &str, planet_name: &str, interval: u64) {
        let host = host.to_string();
        let planet_name = planet_name.to_string();
        
        let mut last_timestamp = time::precise_time_s() as u64;
        let lock = Arc::new(RwLock::new(String::new()));
        let lock_copy = lock.clone();
        
        let (tx, rx) = mpsc::channel();
        thread::spawn(move|| { Curiosity::get_containers(&lock); });
        thread::spawn(move|| { Curiosity::post_containers(&rx, &host, &planet_name); });

        loop {
            let current_timestamp = time::precise_time_s() as u64;
            let diff = current_timestamp - last_timestamp;
            if diff < interval { thread::sleep_ms(1 * 1000); continue; }
            last_timestamp = current_timestamp;
            
            thread::sleep_ms(1);
            let val = match lock_copy.read() {
                Ok(val) => val,
                Err(_) => { continue; }
            };

            match tx.send(val.clone()) {
                Ok(_) => {}
                Err(e) => { println!("{}", e); continue; }
            }
        }
    }
    
    fn get_containers(lock: &RwLock<String>) {
        loop {
            let containers = match container::get_containers_as_str() {
                Ok(containers) => containers,
                Err(e) => { println!("{}", e); continue; }
            };

            thread::sleep_ms(1);
            let mut val = match lock.write() {
                Ok(val) => val,
                Err(_) => { continue; }
            };
            *val = containers;
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
