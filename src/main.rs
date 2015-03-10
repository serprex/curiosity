extern crate unix_socket;
extern crate hyper;
extern crate url;

use std::env;
use std::io::Read;
use std::io::Write;
use std::string::String;
use std::old_io::timer;
use std::time::Duration;

use unix_socket::UnixStream;
use hyper::Client;
use hyper::header::Connection;
use hyper::header::ConnectionOption;
use hyper::header::ContentType;
use hyper::mime::Mime;
use url::form_urlencoded;

fn run(host: &str) {
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
    
    let split: Vec<&str> = response[..].split("\r\n\r\n").collect();
    let containers = split[split.len() - 1];
    
    let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
    let params = vec![("containers", containers)];
    let body = form_urlencoded::serialize(params.into_iter());
    let mut client = Client::new();
    let res = client.post(host)
        .header(Connection(vec![ConnectionOption::Close]))
        .header(ContentType(mime))
        .body(&*body)
        .send();
    match res {
        Ok(_) => {}
        Err(e) => panic!("error http send: {}", e)
    }
}

fn main() {
    let host = match env::var("HOST") {
        Ok(val) => val,
        Err(e) => panic!("error envionment variable: {}", e)
    };
    
    loop {
        run(&host);
        timer::sleep(Duration::seconds(5));
    }
}
