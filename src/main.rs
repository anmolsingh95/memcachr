use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

// request handlers
fn handle_set_request(
    request: SetRequest,
    cache: &mut HashMap<Vec<u8>, SetRequest>,
    mut stream: TcpStream,
) -> std::io::Result<()> {
    log_info("inside set request handler");
    let noreply = request.noreply;
    cache.insert(request.key.clone(), request);
    if noreply {
        return Ok(());
    } else {
        stream.write_all("STORED\r\n".as_bytes()).unwrap();
    }
    Ok(())
}

fn handle_get_request(
    request: GetRequest,
    cache: &mut HashMap<Vec<u8>, SetRequest>,
    mut stream: TcpStream,
) -> std::io::Result<()> {
    log_info("inside get request handler");
    let now = SystemTime::now();
    let unix_time = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    if let Some(value) = cache.get(&request.key) {
        if value.ttl != 0 && value.request_time + value.ttl < unix_time {
            cache.remove(&request.key);
            send_end(stream);
            return Ok(());
        }
        stream
            .write_all(
                format!(
                    "VALUE {key} {flags} {bytes}\r\n",
                    key = str::from_utf8(&request.key).unwrap(),
                    flags = value.flags,
                    bytes = value.bytes,
                )
                .as_bytes(),
            )
            .unwrap();
        stream.write_all(&value.data).unwrap();
        stream.write_all(b"\r\n").unwrap();
        send_end(stream);
    } else {
        send_end(stream);
    }

    Ok(())
}

fn handle_unknown_request(stream: TcpStream) -> std::io::Result<()> {
    log_info("inside unknown request handler");
    send_end(stream);
    Ok(())
}

fn send_end(mut stream: TcpStream) {
    stream.write_all("END\r\n".as_bytes()).unwrap();
}

// Parsing request
#[derive(Debug)]
struct GetRequest {
    key: Vec<u8>,
}

#[derive(Debug)]
struct SetRequest {
    key: Vec<u8>,
    flags: u32,
    ttl: u64,
    bytes: u32,
    noreply: bool,
    data: Vec<u8>,
    request_time: u64,
}

#[derive(Debug)]
enum Request {
    Get(GetRequest),
    Set(SetRequest),
    Unknown,
}

fn parse_request(stream: &mut TcpStream) -> Request {
    let now = SystemTime::now();
    let unix_time = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut reader = BufReader::new(stream);
    let mut command_line = String::new();
    reader.read_line(&mut command_line).unwrap();

    let mut iter = command_line.split_whitespace();
    let command = iter.next().unwrap();
    match command {
        "get" => {
            let key = iter.next().unwrap();
            return Request::Get(GetRequest {
                key: key.as_bytes().to_vec(),
            });
        }
        "set" => {
            let key = iter.next().unwrap();
            let flags = iter.next().unwrap().parse().unwrap();
            let ttl = iter.next().unwrap().parse().unwrap();
            let bytes = iter.next().unwrap().parse().unwrap();
            let noreply = iter.next().unwrap_or("false") == "noreply";
            let mut data_block = String::new();
            reader.read_line(&mut data_block).unwrap();
            return Request::Set(SetRequest {
                key: key.as_bytes().to_vec(),
                flags,
                bytes,
                ttl,
                noreply,
                data: data_block.as_bytes().to_vec(),
                request_time: unix_time,
            });
        }
        _ => Request::Unknown,
    }
}

fn handle_connection(
    mut stream: TcpStream,
    cache: &mut HashMap<Vec<u8>, SetRequest>,
) -> std::io::Result<()> {
    let peer_addr = stream.peer_addr().unwrap();
    log_info(&format!("handling tcpstream from {peer_addr}"));
    let request = parse_request(&mut stream);
    log_info(&format!("received request: {request:?}"));
    match request {
        Request::Get(get_request) => {
            handle_get_request(get_request, cache, stream)
        }
        Request::Set(set_request) => {
            handle_set_request(set_request, cache, stream)
        }
        Request::Unknown => handle_unknown_request(stream),
    }
    .unwrap();
    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:11212").unwrap();
    let local_addr = listener.local_addr().unwrap();
    log_info(&format!("memcachr listening at {local_addr}"));

    let mut cache: HashMap<Vec<u8>, SetRequest> = HashMap::new();
    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_connection(stream.unwrap(), &mut cache).unwrap();
    }
    Ok(())
}

// Utility functions
fn log_info(str: &str) {
    #[cfg(debug_assertions)]
    println!("INFO: {str}");
}
