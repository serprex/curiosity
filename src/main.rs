//! Curiosity
#![doc(html_root_url="https://cosmos-io.github.io/curiosity/doc")]
/*extern crate docker;
extern crate cosmos;

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

use curiosity::Curiosity;*/

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
//use std::env;

fn handle_client(s: TcpStream) {
    let mut stream = match s.try_clone() {
        Ok(stream) => stream,
        Err(e) => { println!("{}", e); return; }
    };

    const BUFFER_SIZE: usize = 1024;
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut raw: Vec<u8> = Vec::new();
    loop {
        let len = match stream.read(&mut buffer) {
            Ok(len) => len,
            Err(_) => { break; }
        };
        for i in 0..len { raw.push(buffer[i]); }
        if len < BUFFER_SIZE { break; }
    }

    let request = match String::from_utf8(raw) {
        Ok(request) => request,
        Err(e) => { println!("{}", e); return; }
    };

    println!("{}", request);

    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8\r\nConnection: close\r\n\r\n";
    let _ = stream.write(response.as_bytes());

    // docker image pull
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8888").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move|| {
                    handle_client(stream)
                });
            }
            Err(e) => { println!("{}", e); }
        }
    }
    drop(listener);
        
    /*let host = match env::var("COSMOS_HOST") {
    // if COSMOS_HOST is present in envrionment variables, Ok(host) will be returned
    // else, Err(_) will be returned
    let host = match env::var("COSMOS_HOST") {
        Ok(host) => host,
        Err(_) => {
            println!("COSMOS_HOST envrionment variable does not exist.");
            println!("127.0.0.1:8888 is used for the cosmos.");
            "127.0.0.1:8888".to_string()
        }
    };*/

    //let interval: u64 = 10;
    //let curiosity = Curiosity::new();
    //curiosity.run(&host, interval);
}

#[test]
fn test() {
    let _ = Curiosity::new();
}
