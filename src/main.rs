//! Curiosity
#![doc(html_root_url="https://cosmos-io.github.io/curiosity/doc")]

// Third party packages
extern crate docker; 
extern crate cosmos; 
extern crate time;

// declare our modules
mod curiosity;
mod container;
mod volume;

// declare our modules
use std::env;
use curiosity::Curiosity;

fn main() {
    // if COSMOS_HOST is present in envrionment variables, Ok(host) will be returned
    // else, Err(_) will be returned
    let host = match env::var("COSMOS_HOST") {
        Ok(host) => host,
        Err(_) => {
            println!("COSMOS_HOST envrionment variable does not exist.");
            println!("127.0.0.1:8888 is used for the cosmos.");
            "127.0.0.1:8888".to_string()
        }
    };

    let interval: u64 = 10;

    // run curiosity
    let curiosity = Curiosity::new();
    curiosity.run(&host, interval);
}

#[test]
fn test() {
    let _ = Curiosity::new();
}
