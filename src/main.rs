#![feature(std_misc)]
#![feature(thread_sleep)]

extern crate docker;
extern crate cosmos;
extern crate rustc_serialize;

use std::env;
use std::time::Duration;
use std::thread;

use docker::Docker;
use docker::stats::Stats;
use docker::container::Port;
use cosmos::Cosmos;
use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
struct Data {
    Id: String,
    Image: String,
    Status: String,
    Command: String,
    Created: f64,
    Names: Vec<String>,
    Ports: Vec<Port>,
    Stats: Stats
}

fn run(host: &str, planet_name: &str) {
    let docker = Docker::new();
    let containers = match docker.get_containers() {
        Ok(containers) => containers,
        Err(e) => { println!("{}", e); return; }
    };

    let mut container_stats: Vec<Data> = Vec::new();
    for container in containers.iter() {
        match docker.get_stats(&container) {
            Err(e) => { println!("{}", e); return; }
            Ok(stats) => {
                let data = Data {
                    Id: container.Id.clone(),
                    Image: container.Image.clone(),
                    Status: container.Status.clone(),
                    Command: container.Command.clone(),
                    Created: container.Created.clone(),
                    Names: container.Names.clone(),
                    Ports: container.Ports.clone(),
                    Stats: stats
                };
                container_stats.push(data);
            }
        };
    }

    let encoded_containers = json::encode(&container_stats).unwrap();
    let cosmos = Cosmos::new(host);
    match cosmos.post_containers(planet_name, &encoded_containers) {
        Ok(res) => { println!("{}", res); }
        Err(e) => { println!("{}", e); }
    };
}

fn main() {
    let host = match env::var("COSMOS_HOST") {
        Ok(host) => host,
        Err(_) => { panic!("COSMOS_HOST envrionment variable does not exist.") }
    };

    let planet_name = match env::var("COSMOS_PLANET_NAME") {
        Ok(planet_name) => planet_name,
        Err(_) => {
            let docker = Docker::new();
            let name = match docker.get_info() {
                Ok(info) => info.Name,
                Err(e) => { panic!("{}", e); }
            };
            name
        }
    };
    
    loop {
        run(&host, &planet_name);
        thread::sleep(Duration::seconds(5));
    }
}
