extern crate mio;
use mio::net::*;
use mio::tcp::Shutdown;
use mio::*;
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;

const ACCEPTABLE: Token = Token(0);

fn main() {
    let addr = "127.0.0.1:8888".parse().expect("addr parse failed");
    let server = TcpListener::bind(&addr).expect("bind failed");
    let poll = Poll::new().expect("accept poll start failed");
    let mut events = Events::with_capacity(1024);
    let mut counter: usize = 10;
    let mut streams: HashMap<usize, TcpStream> = HashMap::new();

    poll.register(&server, ACCEPTABLE, Ready::readable(), PollOpt::edge())
        .expect("failed register server accept");
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(10)))
            .expect("failed poll events");
        for event in events.iter() {
            match event.token() {
                ACCEPTABLE => {
                    let (stream, _) = server.accept().expect("failed accept");
                    stream.set_nodelay(true).expect("failed enable tcp nodelay");
                    poll.register(&stream, Token(counter), Ready::writable(), PollOpt::edge())
                        .expect("failed register stream writable");
                    streams.insert(counter, stream);
                    counter = counter + 1;
                }
                token if token.0 >= 10 && event.readiness().is_writable() => {
                    let stream = streams.remove(&token.0).expect("unexpected stream id");
                    poll.deregister(&stream).expect("deregister writable");
                    send_response(stream);
                }
                _ => unreachable!(),
            }
        }
        events.clear();
    }
}
fn send_response(mut stream: TcpStream) {
    let contents = "HTTP/1.1 200 OK\r\nServer: MIOHttpServer\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 0\r\nConnection: Close\r\n\r\n";
    stream
        .write(contents.as_bytes())
        .expect("failed write bytes");
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => (),
        Err(e) => println!("shutdown failed. {}", e),
    }
}
