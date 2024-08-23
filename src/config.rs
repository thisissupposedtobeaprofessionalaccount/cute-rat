use std::time;

pub struct Config {
    pub server: ServerInfo,
    pub timeout: u64,
    pub request_period: Period,
    pub silent_mode: bool,
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

pub enum TimeUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
}

pub struct Period {
    pub value: u64,
    pub unit: TimeUnit,
}

impl Period {
    pub fn new(value: u64, unit: TimeUnit) -> Self {
        Period { value, unit }
    }
    pub fn to_duration(&self) -> time::Duration {
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
    pub address: String,
    pub port: u16,
}
impl ServerInfo {
    pub fn new(address: &str, port: u16) -> Self {
        ServerInfo {
            address: address.to_string(),
            port,
        }
    }

    pub fn full_address(&self) -> std::net::SocketAddr {
        let ip: std::net::IpAddr = self.address.parse().unwrap();
        std::net::SocketAddr::new(ip, self.port)
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use super::*;

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
