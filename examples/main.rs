extern crate clap;
extern crate examplehttp;
extern crate futures;
#[macro_use]
extern crate serde_derive;
extern crate serde;
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
        .arg(
            Arg::with_name("thread_pool_size")
                .long("thread-pool-size")
                .takes_value(true)
                .default_value("4"),
        )
        .get_matches();
    let mut config: Configuration = Default::default();
    config.bind_port = matches
        .value_of("port")
        .unwrap_or("8888")
        .parse()
        .expect("get bind port");
    config.use_loop = matches.value_of("use_loop").unwrap_or_default() == "true";
    config.thread_pool_size = matches
        .value_of("thread_pool_size")
        .unwrap_or_default()
        .parse()
        .expect("thread pool size");
    info!("config: {:?}", config);
    let mut server = examplehttp::Server::new(config);
    server.with_handler(|request: Request| {
        let response = future::lazy(move || {
            let current_thread = std::thread::current();
            Ok(Response {
                content_type: "application/json".to_owned(),
                body: serde_json::to_string_pretty(&Message {
                    request,
                    ext: Ext {
                        process_thread: format!(
                            "{}:{:?}",
                            current_thread.name().expect("get thread name"),
                            current_thread.id()
                        ),
                    },
                })
                .expect("serialize request"),
            })
        });
        Box::new(response)
    });
    server.start();
}
#[derive(Serialize, Deserialize, Clone)]
struct Message {
    request: examplehttp::Request,
    ext: Ext,
}

#[derive(Serialize, Deserialize, Clone)]
struct Ext {
    process_thread: String,
}
