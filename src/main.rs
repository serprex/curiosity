extern crate unix_socket;
extern crate hyper;
extern crate url;

use std::io::Read;
use std::io::Write;
use std::string::String;

use unix_socket::UnixStream;
use hyper::Client;
use hyper::Url;
use url::form_urlencoded;

fn main() {
    let mut stream = match UnixStream::connect("/var/run/docker.sock") {
        Ok(stream) => stream,
        Err(e) => panic!("error stream connect: {}", e)
    };
    let request = "GET /containers/json HTTP/1.1\r\n\r\n".as_bytes();

    match stream.write_all(request) {
        Ok(_) => {}
        Err(e) => panic!("error stream write: {}", e)
    };

    const BUFFER_SIZE: usize = 1024;
    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut response = String::new();
    loop {
        let len = match stream.read(&mut buf) {
            Ok(len) => len,
            Err(e) => panic!("error stream read: {}", e)
        };

        match std::str::from_utf8(&buf[0 .. len]) {
            Ok(txt) => response.push_str(txt),
            Err(e) => panic!("error stream read: {}", e)
        }
        if len < BUFFER_SIZE { break; }
    }
    
    let split: Vec<&str> = response.as_slice().split("\r\n\r\n").collect();
    let response_body = split[split.len() - 1];
    println!("{}", response_body);
}
