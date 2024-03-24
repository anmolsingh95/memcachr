use std::net::{TcpListener, TcpStream};

fn handle_client(_stream: TcpStream) {
    println!("handling tcpstream");
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:11212").unwrap();
    let local_addr = listener.local_addr().unwrap();
    println!("memcachr listening at {local_addr}");

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}
