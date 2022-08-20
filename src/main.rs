use std::io::{Read, Write, ErrorKind};
use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

const LOCAL_HOST: &str = "127.0.0.1:6000";
const MAX_MSG_LENGTH: u8 = 32;
const THREAD_SLEEP_TIME: u64 = 100;

fn sleep(millis: u64) {
    thread::sleep(std::time::Duration::from_millis(millis))
}

fn listen_for_client_msgs(ip_addr: SocketAddr, sender: Sender<String>) {
    loop {
        let mut buffer = vec![0, MAX_MSG_LENGTH];
        match socket.read_exact(&mut buffer) {
            Ok(_) => {
                let message = buffer
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<_>>();

                let message = String::from_utf8(message)
                    .expect("Invalid UTF-8 formatted message.");

                println!("{}: {:?}", ip_addr, message);

                sender.send(message)
                    .expect("Sender failed to send the message.");
            },
            Err(ref error) if ErrorKind::WouldBlock == error.kind() => {
                continue; // this kind of error isn't important
            }
            Err(_) => {
                println!("closing connection with client at: {}", ip_addr);
                break;
            }
        }

        sleep(THREAD_SLEEP_TIME)
    }
}

fn main() {
    let tcp_listener = TcpListener::bind(LOCAL_HOST)
        .expect("TCP Listener failed to bind.");

    tcp_listener.set_nonblocking(true)
        .expect("Failed to set nonblocking mode for the TCP Listener.");

    let (
        sender,
        receiver
    ) =
        mpsc::channel::<String>();

    let mut client_sockets = vec![];

    loop {
        if let Ok((mut socket, ip_addr)) = tcp_listener.accept() {
            println!("client connected: {}", ip_addr);

            let socket_clone = socket.try_clone()
                .expect("Failed to clone a socket.");

            let sender_clone = sender.clone();

            client_sockets.push(socket_clone);

            thread::spawn(move || {
                listen_for_client_msgs(ip_addr, sender_clone)
            });
        }

        if let Ok(message) = receiver.try_recv() {
            client_sockets = client_sockets.into_iter().filter_map(
                |mut client_socket| {
                    let mut buffer = message.clone().into_bytes();
                    buffer.resize(MAX_MSG_LENGTH as usize, 0);

                    return client_socket.write_all(&buffer)
                        .map(|_| client_socket).ok();
                }
            ).collect::<Vec<_>>()
        }

        sleep(THREAD_SLEEP_TIME)
    }
}
