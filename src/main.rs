extern crate cute_rat;

const PORT: u16 = 7878;
const HOST: &str = "127.0.0.1";

fn main() {
    let full_address = format!("{}:{}", HOST, PORT);
    cute_rat::run(full_address);
}

