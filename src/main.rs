//! Curiosity
#![doc(html_root_url="https://cosmos-io.github.io/curiosity/doc")]
extern crate docker;
extern crate cosmos;
extern crate rustc_serialize;
extern crate time;

mod curiosity;
mod container;
mod volume;

use std::env;
use curiosity::Curiosity;

fn main() {
    let host = match env::var("COSMOS_HOST") {
        Ok(host) => host,
        Err(_) => {
            println!("COSMOS_HOST envrionment variable does not exist.");
            "127.0.0.1:8888".to_string()
        }
    };

    let interval: u64 = 10;

    let curiosity = Curiosity::new();
    curiosity.run(&host, interval);
}

#[test]
fn test() {
    let _ = Curiosity::new();
}
