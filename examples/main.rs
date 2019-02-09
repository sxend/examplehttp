extern crate clap;
extern crate examplehttp;
extern crate futures;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate env_logger;
extern crate timer;
extern crate tokio_timer;

use clap::{App, Arg};
use examplehttp::{Configuration, Request, Response};
use futures::future;
use std::sync::mpsc::channel;

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
        .arg(
            Arg::with_name("sleep")
                .long("sleep")
                .takes_value(true)
                .default_value("0"),
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
    let sleep: u64 = matches
        .value_of("sleep")
        .unwrap_or_default()
        .parse()
        .expect("sleep ms u64");
    info!("config: {:?}", config);
    let mut server = examplehttp::Server::new(config);
    server.with_handler(Box::new(MyHandler { sleep }));
    server.start();
}

struct MyHandler {
    sleep: u64,
}
unsafe impl Send for MyHandler {}
unsafe impl Sync for MyHandler {}

impl examplehttp::Handler for MyHandler {
    fn handle(&self, request: Request) -> examplehttp::BoxFut {
        let sleep = self.sleep;
        let response = future::lazy(move || {
            let timer = timer::Timer::new();
            let (tx, rx) = channel();
            let _guard = timer.schedule_with_delay(
                chrono::Duration::milliseconds(sleep as i64),
                move || {
                    let request = request.clone();
                    let current_thread = std::thread::current();
                    let response = Response {
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
                    };
                    let _ignore = tx.send(Ok(response));
                },
            );
            rx.recv().expect("receive response")
        });
        Box::new(response)
    }
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
