extern crate examplehttp;

fn main() {
    let config = Default::default();
    examplehttp::Server::start(config);
}
