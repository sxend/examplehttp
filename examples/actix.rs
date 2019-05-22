extern crate clap;
extern crate futures;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate actix;
extern crate actix_web;
extern crate env_logger;

use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::Method;
use actix_web::{server, App, HttpRequest, HttpResponse};
use clap::{App as ClapApp, Arg};
use future::FutureResult;
use futures::future;

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
        .arg(
            Arg::with_name("sleep")
                .long("sleep")
                .takes_value(true)
                .default_value("0"),
        )
        .get_matches();
    let bind_port: u32 = matches
        .value_of("port")
        .unwrap_or("8890")
        .parse()
        .expect("get bind port");
    let thread_pool_size: usize = matches
        .value_of("thread_pool_size")
        .unwrap_or("1")
        .parse()
        .expect("get thread_pool_size");
    let _sleep: u64 = matches
        .value_of("sleep")
        .unwrap_or_default()
        .parse()
        .expect("sleep ms u64");
    let _ = actix::System::new("basic-example");
    let _ = server::new(|| App::new().resource("/", |r| r.method(Method::GET).a(handler)))
        .workers(thread_pool_size)
        .bind(format!("0.0.0.0:{}", bind_port))
        .expect("failed to bind")
        .run();
}
fn handler(req: &HttpRequest) -> FutureResult<HttpResponse, actix_web::Error> {
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
        request: ActixRequest {
            version: format!("{:?}", req.version()),
            method: req.method().as_str().to_owned(),
            path: req.path().to_string(),
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
    future::result(Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(
            serde_json::to_string_pretty(&message).expect("serialize request"),
        )))
}

#[derive(Serialize, Deserialize, Clone)]
struct ActixRequest {
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
    request: ActixRequest,
    ext: Ext,
}

#[derive(Serialize, Deserialize, Clone)]
struct Ext {
    process_thread: String,
}
