//! Curiosity
#![doc(html_root_url="https://cosmos-io.github.io/curiosity/doc")]
/*
// Third party packages
extern crate docker;
extern crate cosmos;

// declare our modules
mod curiosity;
mod container;
mod volume;

// declare our modules
use std::env;

use curiosity::Curiosity;*/

extern crate docker;

use std::io::{Read, Write};
use std::env;
use std::error::Error;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::path::Path;
use std::thread;

use docker::Docker;

fn get_docker() -> std::io::Result<Docker> {
    let docker_host = match env::var("DOCKER_HOST") {
        Ok(host) => host,
        Err(_) => "unix:///var/run/docker.sock".to_string() // default address
    };

    let docker_cert_path = match env::var("DOCKER_CERT_PATH") {
        Ok(host) => host,
        Err(_) => "".to_string()
    };

    let mut docker = match docker::Docker::connect(&docker_host) {
        Ok(docker) => docker,
        Err(_) => {
            let err = std::io::Error::new(std::io::ErrorKind::NotConnected,
                                          "The connection is not connected with DOCKER_HOST.");
            return Err(err);
        }
    };

    if docker_cert_path != "" {
        let key = Path::new(&docker_cert_path).join("key.pem");
        let cert = Path::new(&docker_cert_path).join("cert.pem");
        let ca = Path::new(&docker_cert_path).join("ca.pem");
        match docker.set_tls(&key, &cert, &ca) {
            Ok(_) => {},
            Err(_) => {
                let err = std::io::Error::new(std::io::ErrorKind::NotConnected,
                                              "The connection is not connected with DOCKER_CERT_PATH.");
                return Err(err);
            }
        }
    }

    return Ok(docker);
}

fn handle_client(s: TcpStream) -> std::io::Result<()> {
    let mut stream = try!(s.try_clone());

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
        Err(e) => {
            let err = std::io::Error::new(std::io::ErrorKind::InvalidInput,
                                          e.description());
            return Err(err);
        }
    };

    println!("{}", request);

    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8\r\nConnection: close\r\n\r\n";
    let _ = stream.write(response.as_bytes());
    let _ = stream.shutdown(Shutdown::Write); // close a connection

    let mut docker = try!(get_docker());
    let image = "cosmosio/curiosity".to_string();
    let tag = "nightly".to_string();
    let statuses = try!(docker.create_image(image, tag));

    match statuses.last() {
        Some(last) => {
            println!("{}", last.clone().status.unwrap());
        }
        None => { println!("none"); }
    }

    // send a signal to cosmos

    return Ok(());
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8888").unwrap();
    let handle_stream = |stream: TcpStream| {
        thread::spawn(move|| {
            match handle_client(stream) {
                Ok(()) => {}
                Err(e) => { println!("{}", e); }
            }
        });
    };
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => { handle_stream(stream); }
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
    //let _ = Curiosity::new();
}
