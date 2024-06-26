use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

// request handlers
fn handle_set_request(
    request: SetRequest,
    writer: &mut BufWriter<&TcpStream>,
    cache: &mut Cache,
) -> std::io::Result<()> {
    log_info("inside set request handler");
    let noreply = request.noreply;
    let mut cache = cache.lock().unwrap();
    cache.insert(request.key.clone(), request);
    if noreply {
        return Ok(());
    } else {
        writer.write_all("STORED\r\n".as_bytes()).unwrap();
        writer.flush().unwrap();
    }
    Ok(())
}

fn handle_get_request(
    request: GetRequest,
    writer: &mut BufWriter<&TcpStream>,
    cache: &mut Cache,
) -> std::io::Result<()> {
    log_info("inside get request handler");
    let now = SystemTime::now();
    let unix_time = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut cache = cache.lock().unwrap();
    if let Some(value) = cache.get(&request.key) {
        if value.ttl != 0 && value.request_time + value.ttl < unix_time {
            cache.remove(&request.key);
            send_end(writer);
            return Ok(());
        }
        let response_doc = format!(
            "VALUE {key} {flags} {bytes}\r\n",
            key = str::from_utf8(&request.key).unwrap(),
            flags = value.flags,
            bytes = value.bytes
        ) + format!(
            "{data}\r\n",
            data = str::from_utf8(&value.data).unwrap()
        )
        .as_str()
            + "END\r\n";
        writer.write_all(response_doc.as_bytes()).unwrap();
        writer.flush().unwrap();
    } else {
        send_end(writer);
    }

    Ok(())
}

fn handle_unknown_request(
    writer: &mut BufWriter<&TcpStream>,
) -> std::io::Result<()> {
    log_info("inside unknown request handler");
    send_end(writer);
    Ok(())
}

fn send_end(writer: &mut BufWriter<&TcpStream>) {
    writer.write_all("END\r\n".as_bytes()).unwrap();
    writer.flush().unwrap();
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

fn parse_request(
    reader: &mut BufReader<&TcpStream>,
) -> std::io::Result<Request> {
    let now = SystemTime::now();
    let unix_time = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut command_line = String::new();
    reader.read_line(&mut command_line).unwrap();

    let mut iter = command_line.split_whitespace();
    let command = iter.next().unwrap();
    match command {
        "get" => {
            let key = iter.next().unwrap();
            let request = Request::Get(GetRequest {
                key: key.as_bytes().to_vec(),
            });
            Ok(request)
        }
        "set" => {
            let key = iter.next().unwrap();
            let flags = iter.next().unwrap().parse().unwrap();
            let ttl = iter.next().unwrap().parse().unwrap();
            let bytes = iter.next().unwrap().parse().unwrap();
            let noreply = iter.next().unwrap_or("false") == "noreply";
            let mut data_block = String::new();
            reader.read_line(&mut data_block).unwrap();
            let request = Request::Set(SetRequest {
                key: key.as_bytes().to_vec(),
                flags,
                bytes,
                ttl,
                noreply,
                data: data_block.trim().as_bytes().to_vec(),
                request_time: unix_time,
            });
            Ok(request)
        }
        _ => Ok(Request::Unknown),
    }
}

fn handle_request(
    request: Request,
    writer: &mut BufWriter<&TcpStream>,
    cache: &mut Cache,
) {
    log_info(&format!("received request: {request:?}"));
    match request {
        Request::Get(get_request) => {
            handle_get_request(get_request, writer, cache)
        }
        Request::Set(set_request) => {
            handle_set_request(set_request, writer, cache)
        }
        Request::Unknown => handle_unknown_request(writer),
    }
    .unwrap();
    log_info("finished handling last request");
}

fn handle_connection(
    stream: TcpStream,
    mut cache: Cache,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let peer_addr = stream.peer_addr().unwrap();
    log_info(&format!("inbound connection from {peer_addr}"));
    loop {
        log_info(&format!("waiting for next request from {peer_addr}"));
        let request = parse_request(&mut reader);
        match request {
            Ok(request) => {
                handle_request(request, &mut writer, &mut cache);
            }
            Err(_) => {
                log_info(&format!("sending error to {peer_addr}"));
                writer.write_all("ERROR\r\n".as_bytes()).unwrap();
                writer.flush().unwrap();
                break;
            }
        }
    }
    log_info(&format!("closing connection with {peer_addr}"));
    Ok(())
}

type Cache = Arc<Mutex<HashMap<Vec<u8>, SetRequest>>>;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:11212").unwrap();
    let local_addr = listener.local_addr().unwrap();
    log_info(&format!("memcachr listening at {local_addr}"));

    let cache = Arc::new(Mutex::new(HashMap::new()));
    // accept connections and process them serially
    for stream in listener.incoming() {
        let cache = cache.clone();
        thread::spawn(move || {
            handle_connection(stream.unwrap(), cache).unwrap();
        });
    }
    Ok(())
}

fn get_sys_time_in_secs() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_micros(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

// Utility functions
fn log_info(str: &str) {
    let pid = std::process::id();
    let unix_time_us = get_sys_time_in_secs();
    #[cfg(debug_assertions)]
    println!("Cachr: [{pid}] [{unix_time_us}]: {str}");
}
