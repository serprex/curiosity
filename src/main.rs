#![feature(std_misc)]
#![feature(thread_sleep)]
extern crate unix_socket;
extern crate hyper;
extern crate "rustc-serialize" as rustc_serialize;

use std::env;
use std::io::Read;
use std::io::Write;
use std::string::String;
use std::time::Duration;
use std::thread;

use rustc_serialize::json;
use unix_socket::UnixStream;
use hyper::Client;
use hyper::header::Connection;
use hyper::header::ConnectionOption;
use hyper::header::ContentType;
use hyper::header::Accept;
use hyper::header::qitem;
use hyper::mime::Mime;
use hyper::mime::TopLevel::Application;
use hyper::mime::SubLevel::Json;

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
struct Container {
    Id: String,
    Image: String,
    Status: String,
    Command: String,
    Created: f64,
    //Names: Vec<String>,
    //Ports: Vec<String>
}

fn run(host: &str, planet_name: &str) {
    let mut stream = match UnixStream::connect("/var/run/docker.sock") {
        Ok(stream) => stream,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    let request = "GET /containers/json HTTP/1.1\r\n\r\n".as_bytes();

    match stream.write_all(request) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    const BUFFER_SIZE: usize = 1024;
    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut response = String::new();
    loop {
        let len = match stream.read(&mut buf) {
            Ok(len) => len,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        match std::str::from_utf8(&buf[0 .. len]) {
            Ok(txt) => response.push_str(txt),
            Err(e) => {
                println!("{}", e);
                return;
            }
        }
        if len < BUFFER_SIZE { break; }
    }
    
    let split: Vec<&str> = response[..].split("\r\n\r\n").collect();
    let containers = split[split.len() - 1];
    
    let decoded: Vec<Container> = json::decode(containers).unwrap();
    let encoded = json::encode(&decoded).unwrap();
    println!("{}", &*format!("http://{}/{}/containers", host, planet_name));
    println!("{}", encoded);
    
    let mime: Mime = "application/json".parse().unwrap();
    let mut client = Client::new();
    let res = client.post(&*format!("http://{}/{}/containers", host, planet_name))
        .header(Connection(vec![ConnectionOption::Close]))
        .header(ContentType(mime))
        .header(Accept(vec![qitem(Mime(Application, Json, vec![]))]))
        .body(&*encoded)
        .send();
    match res {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            return;
        }
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
