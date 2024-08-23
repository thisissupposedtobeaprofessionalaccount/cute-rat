extern crate cute_rat;

const DEFAULT_SERVER_ADDR : &str = "127.0.0.1";
const DEFAULT_SERVER_PORT : u16 = 6247;
const DEFAULT_TIMEOUT_MS : u64 = 1000;

fn main() {
    let mut config = cute_rat::config::Config::new(
        cute_rat::config::ServerInfo::new(DEFAULT_SERVER_ADDR, DEFAULT_SERVER_PORT),
        DEFAULT_TIMEOUT_MS,
        cute_rat::config::Period::new(1, cute_rat::config::TimeUnit::Seconds),
        false,
    );
    cute_rat::run(&mut config);
}

