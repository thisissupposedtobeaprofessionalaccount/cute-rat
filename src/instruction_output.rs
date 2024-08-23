use std::{io::{self, Write}, net::TcpStream};

pub struct InstructionOutput {
    pub output: String,
}

impl InstructionOutput {
    pub fn send_feedback(&self, stream: &mut TcpStream) -> Result<(), io::Error> {
        stream.write(&self.output.as_bytes())?;
        Ok(())
    }
}
