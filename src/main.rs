use std::io;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Output};

const PORT: u16 = 7878;
const HOST: &str = "127.0.0.1";

fn main() {
    let full_address = format!("{}:{}", HOST, PORT);
    let tcp_listener = TcpListener::bind(full_address);

    match tcp_listener {
        Ok(listener) => {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => handle_stream(stream),
                    Err(_e) => {}
                }
            }
        }
        Err(_e) => {}
    }
}

fn handle_stream(mut stream: TcpStream) {
    let mut command = parse_stream(&stream);
    let output = command.output();
    match output {
        Ok(_) => {
            send_output(&mut stream, &output);
        }
        Err(err) => {
            send_error(&mut stream, &err);
        }
    }
}

fn send_error(stream: &mut TcpStream, error: &io::Error) {
    stream
        .write(&format!("{}", error).as_bytes())
        .expect("Failed to write to stream");
}

fn send_output(stream: &mut TcpStream, output: &Result<Output, io::Error>) {
    let result = match output {
        Ok(output) => {
            format!(
                "\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        }
        Err(e) => format!("Failed to execute command: {}", e),
    };

    stream
        .write(&result.as_bytes())
        .expect("Failed to write to stream");
}

fn parse_stream(stream: &TcpStream) -> Command {
    let mut buffer = [0; 1024];
    stream
        .peek(&mut buffer)
        .expect("Failed to read from stream");

    let input = String::from_utf8_lossy(&buffer[..]);
    let input = input.trim_matches(char::from(0));

    let command = parse_command(&input);

    return command;
}

fn parse_command(command: &str) -> Command {
    let mut cmd;
    let mut parts = command.split_whitespace();

    match parts.next() {
        Some(part) => {
            cmd = Command::new(part);
        }
        None => {
            return Command::new("");
        }
    }

    loop {
        match parts.next() {
            Some(part) => {
                cmd.arg(part);
            }
            None => break,
        }
    }

    return cmd;
}