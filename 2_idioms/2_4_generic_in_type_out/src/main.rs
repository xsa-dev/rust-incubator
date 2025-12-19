use std::net::{IpAddr, SocketAddr};

fn main() {
    let not_found = Error::new("NO_USER")
        .with_status(404)
        .with_message("User not found");

    println!(
        "code: {}, status: {}, message: {}",
        not_found.code(),
        not_found.status(),
        not_found.message()
    );
}

#[derive(Debug)]
pub struct Error {
    code: String,
    status: u16,
    message: String,
}

impl Default for Error {
    #[inline]
    fn default() -> Self {
        Self {
            code: "UNKNOWN".to_string(),
            status: 500,
            message: "Unknown error has happened.".to_string(),
        }
    }
}

impl Error {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            ..Self::default()
        }
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn set_status(&mut self, status: u16) -> &mut Self {
        self.status = status;
        self
    }

    pub fn set_message(&mut self, message: impl Into<String>) -> &mut Self {
        self.message = message.into();
        self
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Default)]
pub struct Server(Option<SocketAddr>);

impl Server {
    pub fn bind(&mut self, addr: impl Into<SocketAddr>) {
        self.0 = Some(addr.into());
    }
}

#[cfg(test)]
mod error_spec {
    use super::*;

    mod builder {
        use super::*;

        #[test]
        fn sets_code_status_and_message() {
            let err = Error::new("NOT_FOUND")
                .with_status(404)
                .with_message("missing user");

            assert_eq!(err.code(), "NOT_FOUND");
            assert_eq!(err.status(), 404);
            assert_eq!(err.message(), "missing user");
        }

        #[test]
        fn mutation_helpers_allow_reuse() {
            let mut err = Error::new("BAD_REQUEST");
            err.set_status(400).set_message("invalid payload");

            assert_eq!(err.code(), "BAD_REQUEST");
            assert_eq!(err.status(), 400);
            assert_eq!(err.message(), "invalid payload");
        }
    }
}

#[cfg(test)]
mod server_spec {
    use super::*;

    mod bind {
        use std::net::{Ipv4Addr, Ipv6Addr};

        use super::*;

        #[test]
        fn sets_provided_address_to_server() {
            let mut server = Server::default();

            server.bind((Ipv4Addr::new(127, 0, 0, 1), 8080));
            assert_eq!(format!("{}", server.0.unwrap()), "127.0.0.1:8080");

            server.bind((Ipv6Addr::LOCALHOST, 9911));
            assert_eq!(format!("{}", server.0.unwrap()), "[::1]:9911");
        }

        #[test]
        fn accepts_various_input_types() {
            let mut server = Server::default();

            server.bind((Ipv4Addr::LOCALHOST, 3030));
            assert_eq!(server.0.unwrap().port(), 3030);

            server.bind(SocketAddr::from(([192, 168, 0, 1], 9090)));
            assert_eq!(
                server.0.unwrap().ip(),
                IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1))
            );
        }
    }
}
