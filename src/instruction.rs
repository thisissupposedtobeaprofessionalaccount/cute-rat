use super::config;
use super::received_command::ReceivedCommand;
use super::received_setting::ReceivedSetting;
use super::instruction_output::InstructionOutput;

use std::io;

pub enum Executable {
    Command(ReceivedCommand),
    Setting(ReceivedSetting),
}

impl Executable {
    pub fn execute(&self, config: &mut config::Config) -> Result<InstructionOutput, io::Error> {
        match self {
            Executable::Command(command) => command.execute(),
            Executable::Setting(setting) => setting.apply(config),
        }
    }
}

