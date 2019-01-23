extern crate clap;
extern crate examplehttp;

use clap::{App, Arg};
use examplehttp::Configuration;

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
    examplehttp::Server::start(config);
}
