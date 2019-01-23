extern crate clap;
extern crate examplehttp;
extern crate futures;
extern crate serde_json;

use clap::{App, Arg};
use examplehttp::{Configuration, Request, Response};
use futures::future;

fn main() {
    let matches = App::new("examplehttp")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true),
        )
        .get_matches();
    let mut config: Configuration = Default::default();
    config.bind_port = matches.value_of("port").unwrap_or("9000").parse().unwrap();
    let mut server = examplehttp::Server::new(config);
    server.with_handler(|request: Request| {
        Box::new(future::ok(Response {
            content_type: "application/json".to_owned(),
            body: serde_json::to_string_pretty(&request).unwrap(),
        }))
    });
    server.start();
}
