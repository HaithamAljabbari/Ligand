use figlet_rs::FIGfont;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::thread;
use std::collections::VecDeque;

fn handle_client(mut stream: TcpStream) {
    println!("[+] Handling connection from: {}", stream.peer_addr().unwrap());
    
    let mut handshake_buf = [0u8; 58]; // Length of the handshake message
    if let Err(e) = stream.read_exact(&mut handshake_buf) {
        println!("[!] Handshake read error: {}", e);
        return;
    }
    
    if let Err(e) = stream.write_all(&handshake_buf) {
        println!("[!] Handshake echo error: {}", e);
        return;
    }
    
    println!("[+] Handshake completed with client");
   
    let mut buffer = [0u8; 128];
    let mut message_queue = VecDeque::new();
    
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, 
            Ok(size) => {
        
                for byte in &buffer[..size] {
                    if *byte == b'\n' {
                        
                        let message: Vec<u8> = message_queue.drain(..).collect();
                        if let Ok(msg_str) = String::from_utf8(message) {
                            println!("{}: {}", stream.peer_addr().unwrap(), msg_str);
                        }
                    } else {
                        message_queue.push_back(*byte);
                    }
                }
            }
            Err(e) => {
                println!("[!] Read error: {}", e);
                break;
            }
        }
    }
    
    println!("[-] Client disconnected: {}", stream.peer_addr().unwrap());
    stream.shutdown(Shutdown::Both).unwrap();
}

fn main() {

    let standard_font = FIGfont::standard().unwrap();
    if let Some(figure) = standard_font.convert("LIGAND") {
        println!("{}", figure);
    }
    
    
    let listener = TcpListener::bind("0.0.0.0:3333").expect("Failed to bind port");
    println!("[+] Server listening on port 3333");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("[+] New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                println!("[!] Connection error: {}", e);
            }
        }
    }
}