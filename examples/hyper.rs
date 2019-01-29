extern crate clap;
extern crate futures;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate http;
extern crate hyper;
extern crate tokio;
extern crate tokio_executor;

use clap::{App as ClapApp, Arg};
use futures::future;
use http::header::{HeaderName, HeaderValue};
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use tokio::runtime;

fn main() {
    env_logger::init();
    log_enabled!(log::Level::Info);
    let matches = ClapApp::new("examplehttp")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("thread_pool_size")
                .long("thread-pool-size")
                .takes_value(true)
                .default_value("4"),
        )
        .get_matches();
    let bind_port: u32 = matches
        .value_of("port")
        .unwrap_or("8891")
        .parse()
        .expect("get bind port");
    let thread_pool_size: usize = matches
        .value_of("thread_pool_size")
        .unwrap_or("4")
        .parse()
        .expect("get thread_pool_size");
    let address = format!("0.0.0.0:{}", bind_port)
        .parse()
        .expect("parse bind_address: {}");

    let server = Server::bind(&address)
        .tcp_nodelay(true)
        .tcp_keepalive(Some(std::time::Duration::from_secs(2)))
        .serve(|| service_fn(service))
        .map_err(|e| eprintln!("server error: {}", e));

    let mut builder: runtime::Builder = tokio::runtime::Builder::new();
    let mut rt = builder
        .core_threads(thread_pool_size)
        .build()
        .expect("build rt");
    let mut entered = tokio_executor::enter().expect("nested tokio::run");
    rt.spawn(server);
    entered
        .block_on(rt.shutdown_on_idle())
        .expect("shutdown cannot error");
}

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn service(req: Request<Body>) -> BoxFut {
    let result = future::lazy(|| Ok(Response::new(Body::from(make_body(req)))));
    Box::new(result)
}
fn make_body(req: Request<Body>) -> String {
    let current_thread = std::thread::current();
    let mut headers = Vec::new();
    for header in req.headers().iter() {
        let name: &HeaderName = header.0;
        let value: &HeaderValue = header.1;
        headers.push(Header {
            name: name.as_str().to_owned(),
            value: value.to_str().expect("header value parse").to_owned(),
        })
    }
    let message = Message {
        request: HyperRequest {
            version: format!("{:?}", req.version()),
            method: req.method().as_str().to_owned(),
            path: req.uri().path().to_owned(),
            headers,
        },
        ext: Ext {
            process_thread: format!(
                "{}:{:?}",
                current_thread.name().expect("get thread name"),
                current_thread.id()
            ),
        },
    };
    serde_json::to_string_pretty(&message).expect("parse json")
}

#[derive(Serialize, Deserialize, Clone)]
struct HyperRequest {
    pub version: String,
    pub method: String,
    pub path: String,
    pub headers: Vec<Header>,
}
#[derive(Serialize, Deserialize, Clone)]
struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    request: HyperRequest,
    ext: Ext,
}

#[derive(Serialize, Deserialize, Clone)]
struct Ext {
    process_thread: String,
}
