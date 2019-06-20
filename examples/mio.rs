extern crate mio;
use mio::net::*;
use mio::tcp::Shutdown;
use mio::*;
use std::collections::HashMap;
use std::io::Write;
use std::thread::JoinHandle;
use std::time::Duration;

const ACCEPTABLE: Token = Token(0);

fn main() {
    let addr = "127.0.0.1:8888".parse().expect("addr parse failed");
    let server = TcpListener::bind(&addr).expect("bind failed");
    let accept_poll = Poll::new().expect("accept poll start failed");
    let writable_poll = Poll::new().expect("writable poll start failed");
    let mut accept_events = Events::with_capacity(1024);
    let mut writable_events = Events::with_capacity(1024);
    let mut counter: usize = 0;
    let mut streams: HashMap<usize, TcpStream> = HashMap::new();
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    accept_poll
        .register(&server, ACCEPTABLE, Ready::readable(), PollOpt::edge())
        .unwrap();
    loop {
        accept_poll
            .poll(&mut accept_events, Some(Duration::from_millis(10)))
            .unwrap();
        for event in accept_events.iter() {
            match event.token() {
                ACCEPTABLE => {
                    let (stream, _) = server.accept().expect("accept failed");
                    println!("get acceptable stream");
                    writable_poll
                        .register(&stream, Token(counter), Ready::all(), PollOpt::edge())
                        .unwrap();
                    streams.insert(counter, stream);
                    counter = counter + 1;
                }
                _ => break,
            }
        }
        writable_poll
            .poll(&mut writable_events, Some(Duration::from_millis(10)))
            .unwrap();
        for event in writable_events.iter() {
            let token = event.token();
            let stream = streams.remove(&token.0).expect("unexpected stream id");
            writable_poll
                .deregister(&stream)
                .expect("deregister writable");
            println!("get writable connection");
            let handle = std::thread::spawn(move || send_response(stream));
            threads.push(handle);
        }
    }
    for handle in threads {
        println!("join!!");
        handle.join().unwrap_or_default();
    }
    server.deregister(&accept_poll).unwrap();
}
fn send_response(mut stream: TcpStream) {
    let c = std::thread::current();
    println!("{:?} {:?}", stream, c.id());
    let contents = "HTTP/1.1 200 OK\r\nServer: MIOHttpServer\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 0\r\nConnection: Close\r\n\r\n";
    stream
        .write(contents.as_bytes())
        .expect("write bytes failed");
    stream.shutdown(Shutdown::Both).expect("shutdown failed");
}
