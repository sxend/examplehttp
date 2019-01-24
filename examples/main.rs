extern crate clap;
extern crate examplehttp;
extern crate futures;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};
use examplehttp::{Configuration, Request, Response};
use futures::future;

fn main() {
    env_logger::init();
    log_enabled!(log::Level::Info);
    let matches = App::new("examplehttp")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("use_loop")
                .long("use-loop")
                .takes_value(true)
                .default_value("true"),
        )
        .get_matches();
    let mut config: Configuration = Default::default();
    config.bind_port = matches
        .value_of("port")
        .unwrap_or("8888")
        .parse()
        .expect("get bind port");
    config.use_loop = matches.value_of("use_loop").unwrap_or_default() == "true";
    info!("config: {:?}", config);
    let mut server = examplehttp::Server::new(config);
    server.with_handler(|request: Request| {
        let response = future::lazy(move || {
            Ok(Response {
                content_type: "application/json".to_owned(),
                body: serde_json::to_string_pretty(&request).expect("serialize request"),
            })
        });
        Box::new(response)
    });
    server.start();
}
