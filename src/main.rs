use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

mod constants {
    pub const LOCAL_HOST: &str = "127.0.0.1:6000";
    pub const MAX_MSG_LENGTH: usize = 32;
    pub const DEFAULT_THREAD_SLEEP_MILLIS: u64 = 100;
}

mod  utils {

    pub fn sleep_for(millis: u64) {
        std::thread::sleep(std::time::Duration::from_millis(millis));
    }
}

mod server {
    use std::net::TcpListener;

    use super::constants;

    pub fn new_tcp_listener() -> TcpListener {
        let tcp_listener = TcpListener::bind(constants::LOCAL_HOST)
            .expect("TCP listener failed to bind.");

        tcp_listener.set_nonblocking(true)
            .expect("Failed to set non-blocking flag for the TCP listener");

        return tcp_listener;
    }
}

fn main() {
    let tcp_listener = server::new_tcp_listener();
    let (sender, receiver) = mpsc::channel::<String>();
    let mut client_tcp_streams: Vec<TcpStream> = vec![];

    println!("waiting for clients to connect...");

    loop {
        if let Ok((mut client_tcp_stream, addr)) = tcp_listener.accept() {
            println!("client connected: {}", addr);

            let client_tcp_stream_clone = client_tcp_stream.try_clone()
                .expect("Failed to clone the client's TCP stream.");

            let sender_clone = sender.clone();

            client_tcp_streams.push(client_tcp_stream_clone);

            thread::spawn(move || loop {
                let mut buff = vec![0; constants::MAX_MSG_LENGTH];

                match client_tcp_stream.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        println!("{}: {:?}", addr, msg);
                        sender_clone.send(msg).expect("failed to send msg to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("closed connection with: {}", addr);
                        break;
                    }
                }

                utils::sleep_for(constants::DEFAULT_THREAD_SLEEP_MILLIS);
            });
        }

        if let Ok(message) = receiver.try_recv() {
            client_tcp_streams = client_tcp_streams.into_iter().filter_map(
                |mut client_tcp_stream| {
                    let mut buffer = message.clone().into_bytes();
                    buffer.resize(constants::MAX_MSG_LENGTH, 0);

                    client_tcp_stream
                        .write_all(&buffer)
                        .map(|_| client_tcp_stream)
                        .ok()
                }
            ).collect::<Vec<_>>();
        }

        utils::sleep_for(constants::DEFAULT_THREAD_SLEEP_MILLIS);
    }
}
