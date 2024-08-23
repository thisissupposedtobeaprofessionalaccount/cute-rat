use super::instruction_output::InstructionOutput;
use super::config;

use std::io;


pub struct ReceivedSetting {
    key: String,
    value: String,
}

impl ReceivedSetting {
    pub fn from_string(key: String, value: String) -> Self {
        ReceivedSetting { key, value }
    }

    pub fn parse_period(&self) -> Result<config::Period, std::io::Error> {
        let parts: Vec<&str> = self.value.split_whitespace().collect();

        let value = parts.get(0).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid period value",
        ))?;

        let value = match value.parse::<u64>() {
            Ok(value) => value,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid period value",
                ))
            }
        };

        let unit = *parts.get(1).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid period unit",
        ))?;
        
        let unit = match unit {
            "ms" => config::TimeUnit::Milliseconds,
            "s" => config::TimeUnit::Seconds,
            "m" => config::TimeUnit::Minutes,
            "h" => config::TimeUnit::Hours,
            "d" => config::TimeUnit::Days,
            _ => config::TimeUnit::Seconds,
        };

        Ok(config::Period::new(value, unit))
    }

    pub fn parse_server_info(&self) -> Result<config::ServerInfo, std::io::Error> {
        let parts: Vec<&str> = self.value.split_whitespace().collect();

        let address = parts.get(0).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid server address",
        ))?;

        let port = parts.get(1).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid server port",
        ))?;

        let port = match port.parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid port value",
                ))
            }
        };
        Ok(config::ServerInfo::new(address, port))
    }

    pub fn parse_timeout(&self) -> Result<u64, std::io::Error> {
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

    pub fn apply_setting(&self, config: &mut config::Config) -> Result<(), std::io::Error> {
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

    pub fn apply(&self, config: &mut config::Config) -> Result<InstructionOutput, std::io::Error> {
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

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_received_setting_parse_period() {
        let setting = ReceivedSetting::from_string("period".to_string(), "1 s".to_string());
        let period = setting.parse_period().unwrap();

        assert_eq!(period.value, 1);
        assert!(matches!(period.unit, config::TimeUnit::Seconds));
    }

    #[test]
    fn test_received_setting_parse_period_empty_input() {
        let setting = ReceivedSetting::from_string("period".to_string(), "".to_string());
        let result = setting.parse_period();

        assert!(result.is_err());
    }

    #[test]
    fn test_received_setting_parse_period_missing_unit() {
        let setting = ReceivedSetting::from_string("period".to_string(), "1".to_string());
        let result = setting.parse_period();

        assert!(result.is_err());
    }

    #[test]
    fn test_received_setting_parse_period_bad_input () {
        let setting = ReceivedSetting::from_string("period".to_string(), "rat x".to_string());
        let result = setting.parse_period();

        assert!(result.is_err());
    }

    #[test]
    fn test_received_setting_parse_server_info() {
        let setting = ReceivedSetting::from_string("server".to_string(), "127.0.0.1 6247".to_string());

        let server_info = setting.parse_server_info().unwrap();

        assert_eq!(server_info.address, "127.0.0.1");
        assert_eq!(server_info.port, 6247);
    }

    #[test]
    fn test_received_setting_parse_server_info_empty_input() {
        let setting = ReceivedSetting::from_string("server".to_string(), "".to_string());
        let result = setting.parse_server_info();

        assert!(result.is_err());
    }

    #[test]
    fn test_received_setting_parse_server_info_missing_port() {
        let setting = ReceivedSetting::from_string("server".to_string(), "127.0.0.1".to_string());
        let result = setting.parse_server_info();

        assert!(result.is_err());
    }
    
    #[test]
    fn test_received_setting_parse_server_info_bad_input() {
        let setting = ReceivedSetting::from_string("server".to_string(), "rat rat".to_string());
        let result = setting.parse_server_info();

        assert!(result.is_err());
    }

    #[test]
    fn test_received_setting_timeout() {
        let setting = ReceivedSetting::from_string("timeout".to_string(), "1000".to_string());
        let timeout = setting.parse_timeout().unwrap();

        assert_eq!(timeout, 1000);
    }

    #[test]
    fn test_received_setting_timeout_empty_input() {
        let setting = ReceivedSetting::from_string("timeout".to_string(), "".to_string());
        let result = setting.parse_timeout();

        assert!(result.is_err());
    }
}
