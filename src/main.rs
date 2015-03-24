#![feature(std_misc)]
#![feature(thread_sleep)]

extern crate docker;
extern crate hyper;
extern crate "rustc-serialize" as rustc_serialize;

use std::env;
use std::time::Duration;
use std::thread;

use docker::Docker;
use hyper::Client;
use hyper::header::Connection;
use hyper::header::ConnectionOption;

use hyper::header::ContentType;
use hyper::header::Accept;
use hyper::header::qitem;
use hyper::mime::Mime;
use hyper::mime::TopLevel::Application;
use hyper::mime::SubLevel::Json;
use rustc_serialize::json;

fn run(host: &str, planet_name: &str) {
    let docker = Docker::new();
    let containers = match docker.get_containers() {
        Ok(containers) => containers,
        Err(e) => { println!("{}", e); return; }
    };

    let encoded_containers = json::encode(&containers).unwrap();
    let mime: Mime = "application/json".parse().unwrap();
    let mut client = Client::new();
    let res = client.post(&*format!("http://{}/{}/containers", host, planet_name))
        .header(Connection(vec![ConnectionOption::Close]))
        .header(ContentType(mime))
        .header(Accept(vec![qitem(Mime(Application, Json, vec![]))]))
        .body(&*encoded_containers)
        .send();
    match res {
        Ok(_) => { println!("{}", encoded_containers); }
        Err(e) => { println!("{}", e); }
    }
}

fn main() {
    let host = match env::var("COSMOS_HOST") {
        Ok(host) => host,
        Err(_) => panic!("COSMOS_HOST envrionment variable does not exist.")
    };

    let planet_name = match env::var("COSMOS_PLANET_NAME") {
        Ok(planet_name) => planet_name,
        Err(_) => panic!("COSMOS_PLANET_NAME variable does not exist.")
    };
    
    loop {
        run(&host, &planet_name);
        thread::sleep(Duration::seconds(5));
    }
}
