extern crate http;
extern crate httparse;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate tokio_net;
#[macro_use]
extern crate log;

use futures::future;
use std::sync::Arc;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

#[derive(Debug, Clone)]
pub struct Configuration {
    pub bind_host: String,
    pub bind_port: u32,
    pub use_loop: bool,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            bind_host: "0.0.0.0".to_owned(),
            bind_port: 9000,
            use_loop: true,
        }
    }
}
impl Configuration {}

pub struct Server {
    config: Configuration,
    handler: Arc<Handler>,
}

type Handler = fn(Request) -> Box<Future<Item = Response, Error = std::io::Error> + Send>;

impl Server {
    pub fn new(config: Configuration) -> Self {
        Server {
            config,
            handler: Arc::new(NO_HANDLER),
        }
    }
    pub fn with_handler(&mut self, handler: Handler) {
        self.handler = Arc::new(handler);
    }
    pub fn start(&self) {
        let bind_address = format!("{}:{}", self.config.bind_host, self.config.bind_port);
        let address = bind_address
            .parse()
            .unwrap_or_else(|_| panic!("parse bind_address: {}", bind_address));
        let listener = TcpListener::bind(&address).unwrap_or_else(|_| panic!("bind: {}", address));

        let handler = self.handler.clone();
        let use_loop = self.config.use_loop;
        let server = listener
            .incoming()
            .map_err(|e| error!("accept {:?}", e))
            .for_each(move |stream| {
                handle_stream(stream, handler.clone(), use_loop);
                Ok(())
            });
        tokio::run(server);
    }
}

fn handle_stream(stream: TcpStream, handler: Arc<Handler>, use_loop: bool) {
    let request = if use_loop {
        read_request_loop(stream)
    } else {
        read_request(stream)
    };
    let result = request
        .map_err(|e| error!("read {:?}", e))
        .and_then(move |(stream, request)| {
            let response = handler(request);
            response
                .map_err(|e| error!("body {:?}", e))
                .and_then(|response| {
                    let message = stringify_response(response);
                    io::write_all(stream, message)
                        .map_err(|e| error!("write {:?}", e))
                        .and_then(move |_| Ok(()))
                })
        });

    tokio::spawn(result);
}

const NO_HANDLER: Handler = |_| {
    Box::new(future::ok(Response {
        content_type: "text/plain".to_owned(),
        body: "no handler".to_owned(),
    }))
};

fn stringify_response(response: Response) -> String {
    return format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
        response.content_type,
        response.body.len(),
        response.body
    );
}
fn read_request(
    stream: TcpStream,
) -> Box<Future<Item = (TcpStream, Request), Error = std::io::Error> + Send> {
    let result =
        io::read(stream, vec![0; 1024]).and_then(move |(stream, bytes, size)| match parse_request(
            &bytes[..size],
            20,
        ) {
            ParseResult::Complete(request) => Ok((stream, request)),
            ParseResult::Partial(_) => panic!("request message too large!!"),
            ParseResult::Err(e) => panic!(e),
        });
    Box::new(result)
}
fn read_request_loop(
    stream: TcpStream,
) -> Box<Future<Item = (TcpStream, Request), Error = std::io::Error> + Send> {
    let result = future::loop_fn(
        (stream, Vec::new(), 20),
        move |(stream, mut buf, header_size)| {
            io::read(stream, vec![0; 1024]).and_then(move |(stream, bytes, size)| {
                buf.extend_from_slice(&bytes[..size]);
                match parse_request(&buf, header_size) {
                    ParseResult::Complete(request) => Ok(future::Loop::Break((stream, request))),
                    ParseResult::Partial(new_header_size) => {
                        Ok(future::Loop::Continue((stream, buf, new_header_size)))
                    }
                    ParseResult::Err(e) => panic!(e),
                }
            })
        },
    );
    Box::new(result)
}
enum ParseResult {
    Complete(Request),
    Partial(usize),
    Err(httparse::Error),
}
fn parse_request(buf: &[u8], header_size: usize) -> ParseResult {
    let mut parser = httparse::Request {
        method: None,
        path: None,
        version: None,
        headers: &mut vec![httparse::EMPTY_HEADER; header_size],
    };
    match parser.parse(buf) {
        Ok(result) => {
            if result.is_complete() {
                ParseResult::Complete(convert_request(parser))
            } else {
                ParseResult::Partial(header_size)
            }
        }
        Err(httparse::Error::TooManyHeaders) => parse_request(buf, header_size + 10),
        Err(e) => ParseResult::Err(e),
    }
}
fn convert_request(request: httparse::Request) -> Request {
    let own_headers = &mut Vec::new();
    for header in request.headers {
        own_headers.push(Header {
            name: header.name.to_owned(),
            value: String::from_utf8(header.value.to_vec()).expect("parse header value"),
        })
    }
    Request {
        version: request.version.expect("parse version").to_owned(),
        method: request.method.expect("parse method").to_owned(),
        path: request.path.expect("parse path").to_owned(),
        headers: own_headers.to_owned(),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Request {
    pub version: u8,
    pub method: String,
    pub path: String,
    pub headers: Vec<Header>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Response {
    pub content_type: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}
