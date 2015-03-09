use std::old_io::net::pipe::UnixStream;
use std::string::String;

fn main() {
    let server = Path::new("/var/run/docker.sock");
    let mut stream = UnixStream::connect(&server);
    let request = "GET /containers/json HTTP/1.1\r\n\r\n";
    match stream.write_str(request.as_slice()) {
        Ok(_) => {}
        Err(e) => panic!("error stream write: {}", e)
    };

    const BUFFER_SIZE: usize = 1024;
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut response = String::new();
    loop {
        let len = match stream.read(&mut buffer[0 .. BUFFER_SIZE - 1]) {
            Ok(len) => len,
            Err(e) => panic!("error stream read: {}", e)
        };
        match std::str::from_utf8(&buffer[0 .. len]) {
            Ok(v) => response.push_str(v),
            Err(e) => panic!("error stream read: {}", e)
        }
        if len < BUFFER_SIZE - 1 { break; }
    }

    println!("{}", response);
}
