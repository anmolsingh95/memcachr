use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::str;

fn log_info(str: &str) {
    #[cfg(debug_assertions)]
    println!("INFO: {str}");
}

fn read_until_excluding_delimiter(
    reader: &mut BufReader<impl std::io::Read>,
    delimiter: u8,
    buf: &mut Vec<u8>,
) -> std::io::Result<usize> {
    let bytes_read = reader.read_until(delimiter, buf)?;
    if buf.ends_with(&[delimiter]) {
        buf.pop();
    }
    Ok(bytes_read)
}

fn handle_client(stream: TcpStream) -> std::io::Result<()> {
    let peer_addr = stream.peer_addr().unwrap();
    log_info(&format!("handling tcpstream from {peer_addr}"));

    let mut reader = BufReader::new(&stream);
    let mut buffer = Vec::new();
    read_until_excluding_delimiter(&mut reader, b' ', &mut buffer).unwrap();
    let command = str::from_utf8(&buffer).unwrap();
    log_info(&format!("received command: '{command}'"));
    match command {
        "get" => {
            log_info("get command");
            handle_get_command(stream)
        }
        "set" => {
            log_info("set command");
            handle_set_command(stream)
        }
        _ => {
            log_info("unkonwn command");
            handle_unknown_command(stream)
        }
    }
    .unwrap();

    Ok(())
}

fn handle_set_command(_stream: TcpStream) -> std::io::Result<()> {
    log_info("inside set command handler");
    Ok(())
}

fn handle_get_command(stream: TcpStream) -> std::io::Result<()> {
    log_info("inside get command handler");
    let mut reader = BufReader::new(&stream);
    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;

    log_info(&format!("received buffer inside get: {buffer:?}"));
    let mut split_buffer = buffer.split_whitespace();
    let _command = split_buffer.next().unwrap();
    Ok(())
}

fn handle_unknown_command(mut stream: TcpStream) -> std::io::Result<()> {
    log_info("inside unknown command handler");
    stream.write_all("End\r\n".as_bytes()).unwrap();
    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:11212").unwrap();
    let local_addr = listener.local_addr().unwrap();
    log_info(&format!("memcachr listening at {local_addr}"));

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream?).unwrap();
    }
    Ok(())
}
