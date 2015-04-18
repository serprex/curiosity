extern crate docker;
extern crate cosmos;
extern crate rustc_serialize;
extern crate time;

mod curiosity;
mod container;

use std::env;
use curiosity::Curiosity;

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

    let curiosity = Curiosity::new();
    curiosity.run(&host, &planet_name, interval);
}

#[test]
fn test() {
    let _ = Curiosity::new();
}
