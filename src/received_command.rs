use std::process::{Child, Stdio};

use super::instruction_output::InstructionOutput;

pub struct ReceivedCommand {
    command: String,
}

impl ReceivedCommand {
    pub fn from_string(command: String) -> Self {
        ReceivedCommand { command }
    }

    pub fn parse(&self) -> Result<Vec<std::process::Command>, std::io::Error> {
        let mut cmds = self.command.trim().split("|");

        let mut result = vec![];
        let mut current_command;

        match cmds.next() {
            Some(cmd) => {
                current_command = ReceivedCommand::parse_command_section(cmd.to_string());
            }
            None => return Ok(vec![]),
        }

        loop {
            match cmds.next() {
                Some(cmd) if cmd.is_empty() => break,
                Some(cmd) => {
                    current_command.stdout(Stdio::piped());
                    result.push(current_command);

                    current_command = ReceivedCommand::parse_command_section(cmd.to_string());
                    current_command.stdin(Stdio::piped());
                }
                None => break,
            }
        }

        result.push(current_command);

        Ok(result)
    }

    pub fn parse_command_section(command: String) -> std::process::Command {
        let mut cmd;
        let mut parts = command.split_whitespace();

        match parts.next() {
            Some(part) => {
                cmd = std::process::Command::new(part);
            }
            None => {
                return std::process::Command::new("");
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
        let mut commands = self.parse();

        if let Ok(ref mut commands) = commands {
            return ReceivedCommand::execute_commands(commands);
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Parsing error",
        ))
    }

    fn execute_commands(
        commands: &mut Vec<std::process::Command>,
    ) -> Result<InstructionOutput, std::io::Error> {
        let mut last_command;

        match commands.pop() {
            Some(command) => {
                last_command = command;
            }
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Last command is empty",
                ));
            }
        }

        let mut cmds = commands.iter_mut();


        let mut previous_process: Option<Child> = None;

        loop {
            match cmds.next() {
                Some(command) => {
                    match previous_process {
                        Some(ref mut prev_proc) => {
                            previous_process = command.stdin(prev_proc.stdout.take().unwrap()).spawn().ok();
                        }
                        None => previous_process = command.spawn().ok(), 
                    }
                }
                None => break,
            }
        }

        if let Some(ref mut previous_process) = previous_process {
            last_command.stdin(previous_process.stdout.take().unwrap());
        }

        let output = last_command.output();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_with_just_command() {
        let result = ReceivedCommand::parse_command_section("ls".to_string());

        assert_eq!(result.get_program(), "ls");
        assert_eq!(result.get_args().count(), 0);
    }

    #[test]
    fn test_parse_command_with_one_arg() {
        let result = ReceivedCommand::parse_command_section("ls -l".to_string());

        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-l"]);
    }

    #[test]
    fn test_parse_command_with_multiple_args_one_dash() {
        let result = ReceivedCommand::parse_command_section("ls -la".to_string());
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-la"]);
    }

    #[test]
    fn test_parse_command_with_multiple_args_multiple_dashes() {
        let result = ReceivedCommand::parse_command_section("ls -l -a".to_string());
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-l", "-a"]);
    }

    #[test]
    fn test_parse_command_with_empty_command() {
        let result = ReceivedCommand::parse_command_section("".to_string());

        assert_eq!(result.get_program(), "");
        assert_eq!(result.get_args().count(), 0);
    }

    #[test]
    fn test_parse_command_with_whitespace_command() {
        let result =
            ReceivedCommand::parse_command_section("   ls      -l      -a     ".to_string());
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-l", "-a"]);
    }

    #[test]
    fn test_parse_simple_command() {
        let command = ReceivedCommand::from_string("ls".to_string());
        let commands = command.parse();
        let commands = commands.unwrap();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].get_program(), "ls");
        assert_eq!(commands[0].get_args().count(), 0);
    }

    #[test]
    fn test_parse_simple_command_with_args() {
        let command = ReceivedCommand::from_string("ls -l".to_string());
        let commands = command.parse();
        let commands = commands.unwrap();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].get_program(), "ls");
        assert_eq!(commands[0].get_args().count(), 1);
    }

    #[test]
    fn test_parse_few_piped_commands() {
        let command = ReceivedCommand::from_string("ls -l | grep test".to_string());
        let commands = command.parse();
        let commands = commands.unwrap();

        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0].get_program(), "ls");
        assert_eq!(commands[0].get_args().count(), 1);
        assert_eq!(commands[1].get_program(), "grep");
        assert_eq!(commands[1].get_args().count(), 1);
    }

    #[test]
    fn test_parse_many_piped_commands() {
        let command = ReceivedCommand::from_string("ls -l | grep test | wc -l".to_string());
        let commands = command.parse();
        let commands = commands.unwrap();

        assert_eq!(commands.len(), 3);

        assert_eq!(commands[0].get_program(), "ls");
        assert_eq!(commands[0].get_args().count(), 1);
        assert_eq!(commands[1].get_program(), "grep");
        assert_eq!(commands[1].get_args().count(), 1);
        assert_eq!(commands[2].get_program(), "wc");
        assert_eq!(commands[2].get_args().count(), 1);
    }

    #[test]
    fn test_parse_badly_piped_commands() {
        let command = ReceivedCommand::from_string("ls -l | grep test |".to_string());
        let commands = command.parse();
        let commands = commands.unwrap();

        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0].get_program(), "ls");
        assert_eq!(commands[0].get_args().count(), 1);
        assert_eq!(commands[1].get_program(), "grep");
        assert_eq!(commands[1].get_args().count(), 1);
    }
}
