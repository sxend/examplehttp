extern crate mio;
use mio::*;
use mio::net::*;
use mio::tcp::Shutdown;
use std::io::Write;

const ACCEPTABLE: Token = Token(0);
const WRITABLE: Token = Token(1);

fn main() {
    let addr = "127.0.0.1:8888".parse().expect("addr parse failed");
    let server = TcpListener::bind(&addr).expect("bind failed");
    let mut poll = Poll::new().expect("poll start failed");
    poll.register(
        &server,
        ACCEPTABLE,
        Ready::readable(),
        PollOpt::edge()).unwrap();
    let mut events = Events::with_capacity(1024);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                ACCEPTABLE => {
                    let (stream, addr)  = server.accept().expect("accept failed");
                    std::thread::spawn(move || execute(stream));
                }
                _ => {}
            }
        }
    }
}
fn execute(mut stream: TcpStream) {
    println!("{:?}", stream);
    let contents = "HTTP/1.1 200 OK\r\nServer: MIOHttpServer\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 0\r\nConnection: Close\r\n\r\n";
    stream.write(contents.as_bytes()).expect("write bytes failed");
    stream.shutdown(Shutdown::Both).expect("shutdown failed");
}
struct Socket {
    port: usize
}