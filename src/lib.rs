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
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
}
impl Server {
    pub fn start(config: Configuration) {
        let bind_address = format!("{}:{}", config.bind_host, config.bind_port);
        let address = bind_address
            .parse()
            .expect(&format!("parse bind_address: {}", bind_address));
        let socket = TcpListener::bind(&address).expect(&format!("bind: {}", address));

        let server = socket
            .incoming()
            .map_err(|e| eprintln!("accept {:?}", e))
            .for_each(|stream| {
                let result = io::read(stream, vec![0; 1024])
                    .map_err(|e| eprintln!("read {:?}", e))
                    .and_then(move |(stream, bytes, size)| {
                        let request = Server::parse_request(&bytes[..size]);
                        Ok((stream, request))
                    })
                    .and_then(move |(stream, request)| {
                        let body = serde_json::to_string_pretty(&request).expect("print json");
                        let message = response(body);
                        io::write_all(stream, message)
                            .map_err(|e| eprintln!("write {:?}", e))
                            .and_then(move |_| Ok(()))
                    });
                tokio::spawn(result)
            });
        tokio::run(server);
    }
    fn parse_request<'a>(bytes: &[u8]) -> Request {
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
}

#[derive(Serialize, Deserialize, Clone)]
struct Request {
    version: u8,
    method: String,
    path: String,
    headers: Vec<Header>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Header {
    name: String,
    value: String,
}
