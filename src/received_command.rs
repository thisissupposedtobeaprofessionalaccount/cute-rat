use super::instruction_output::InstructionOutput;

pub struct ReceivedCommand {
    command: String,
}

impl ReceivedCommand {
    pub fn from_string(command: String) -> Self {
        ReceivedCommand { command }
    }
    pub fn parse(&self) -> std::process::Command {
        let mut cmd;
        let mut parts = self.command.split_whitespace();

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
        let command = ReceivedCommand::from_string("   ls      -l      -a     ".to_string());
        let result = command.parse();
        let args: Vec<&std::ffi::OsStr> = result.get_args().collect();

        assert_eq!(result.get_program(), "ls");
        assert_eq!(args, &["-l", "-a"]);
    }
}
