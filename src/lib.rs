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

use futures;
use futures::future::FutureResult;
use std::sync::Arc;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
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

pub struct Server {
    config: Configuration,
    handler: Arc<Option<Handler>>,
}

type Handler = fn(Request) -> Box<FutureResult<Response, std::io::Error>>;

impl Server {
    pub fn new(config: Configuration) -> Self {
        Server {
            config,
            handler: Arc::new(None),
        }
    }
    pub fn with_handler(&mut self, handler: Handler) {
        self.handler = Arc::new(Some(handler));
    }
    pub fn start(&self) {
        let bind_address = format!("{}:{}", self.config.bind_host, self.config.bind_port);
        let address = bind_address
            .parse()
            .expect(&format!("parse bind_address: {}", bind_address));
        let listener = TcpListener::bind(&address).expect(&format!("bind: {}", address));

        let handler = self.handler.clone();
        let server = listener
            .incoming()
            .map_err(|e| eprintln!("accept {:?}", e))
            .for_each(move |stream| {
                StreamHandler::new(stream, handler.clone()).handle();
                Ok(())
            });
        tokio::run(server);
    }
}

struct StreamHandler {
    stream: TcpStream,
    handler: Arc<Option<Handler>>,
}

impl StreamHandler {
    fn new(stream: TcpStream, handler: Arc<Option<Handler>>) -> StreamHandler {
        StreamHandler { stream, handler }
    }
    fn handle(self) {
        let stream = self.stream;
        let handler = self.handler.clone();
        let result = io::read(stream, vec![0; 1024])
            .map_err(|e| eprintln!("read {:?}", e))
            .and_then(|(stream, bytes, size)| {
                let request = parse_request(&bytes[..size]);
                Ok((stream, request))
            })
            .and_then(move |(stream, request)| {
                let body = match *handler {
                    Some(handler) => handler(request),
                    None => Box::new(futures::future::ok(no_handler())),
                };
                body.map_err(|e| eprintln!("body {:?}", e))
                    .and_then(|response| {
                        let message = stringify_response(response);
                        io::write_all(stream, message)
                            .map_err(|e| eprintln!("write {:?}", e))
                            .and_then(move |_| Ok(()))
                    })
            });
        tokio::spawn(result);
    }
}

fn no_handler() -> Response {
    Response {
        content_type: "text/plain".to_owned(),
        body: "no handler".to_owned(),
    }
}

fn stringify_response(response: Response) -> String {
    return format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
        response.content_type,
        response.body.len(),
        response.body
    );
}

fn parse_request(bytes: &[u8]) -> Request {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut request: httparse::Request = httparse::Request::new(&mut headers);
    request.parse(bytes).expect("parse http bytes");
    let own_headers = &mut Vec::new();
    for h in request.headers {
        own_headers.push(Header {
            name: h.name.to_owned(),
            value: String::from_utf8(h.value.to_vec()).expect("parse header value"),
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
