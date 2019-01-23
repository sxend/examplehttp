extern crate http;
extern crate httparse;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate tokio_net;

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

pub struct Configuration {
    pub bind_host: String,
    pub bind_port: u32,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            bind_host: "0.0.0.0".to_owned(),
            bind_port: 9000,
        }
    }
}
impl Configuration {}

pub struct Server {}

fn response(body: String) -> String {
    return format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
}
impl Server {
    pub fn start(config: Configuration) {
        let bind_address = format!("{}:{}", config.bind_host, config.bind_port);
        let address = bind_address.parse().unwrap();
        let socket = TcpListener::bind(&address).unwrap();

        let server = socket
            .incoming()
            .map_err(|e| eprintln!("failed to accept {:?}", e))
            .for_each(|stream| {
                let result = io::read(stream, vec![0; 1024])
                    .map_err(|e| eprintln!("failed to read {:?}", e))
                    .and_then(move |(stream, bytes, size)| {
                        let mut headers = [httparse::EMPTY_HEADER; 16];
                        let mut parser = httparse::Request::new(&mut headers);
                        parser.parse(&bytes[..size]).unwrap();
                        let request = Server::convert(parser);
                        Ok((stream, request))
                    })
                    .and_then(move |(stream, request)| {
                        let message = response(format!("{:?}", request));
                        io::write_all(stream, message)
                            .map_err(|e| eprintln!("failed to write {:?}", e))
                            .and_then(move |_| Ok(()))
                    });

                tokio::spawn(result)
            });
        tokio::run(server);
    }
    fn convert(parser: httparse::Request) -> http::Request<()> {
        let mut builder = &mut http::Request::builder();
        builder = builder.method(parser.method.unwrap());
        for h in parser.headers {
            builder = builder.header(h.name, String::from_utf8(h.value.to_vec()).unwrap());
        }
        builder.body(()).unwrap()
    }
}
