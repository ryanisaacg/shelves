use std::{
    collections::{HashMap, HashSet},
    io::{self, Read, Write},
    net::TcpStream,
    sync::Arc,
};

use rustls::{pki_types::InvalidDnsNameError, ClientConfig};
use thiserror::Error;

use crate::url::{Scheme, Url};

pub struct Client {
    config: Arc<ClientConfig>,
    static_hosts: HashSet<&'static str>,
}

impl Client {
    pub fn new() -> Client {
        let root_store =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth(),
        );

        Client {
            config,
            static_hosts: HashSet::new(),
        }
    }

    pub fn request(&mut self, url: &Url) -> Result<String, RequestError> {
        let host = url.host();

        let mut response = String::new();
        match url.scheme() {
            Scheme::Http => {
                let mut stream = TcpStream::connect((host, url.port().unwrap_or(80)))?;

                self.send_get(url, &mut stream)?;
                stream.read_to_string(&mut response)?;
            }
            Scheme::Https => {
                let static_host = self.static_host(host);
                let mut client =
                    rustls::ClientConnection::new(self.config.clone(), static_host.try_into()?)?;

                let mut stream = TcpStream::connect((host, url.port().unwrap_or(443)))?;
                let mut stream = rustls::Stream::new(&mut client, &mut stream);

                self.send_get(url, &mut stream)?;
                stream.read_to_string(&mut response)?;
            }
            Scheme::File => {
                return Ok(std::fs::read_to_string(url.path())?);
            }
        }

        let mut lines = response.split("\r\n");
        let statusline = lines.next().ok_or(RequestError::UnexpectedEndOfInput)?;

        let mut status_compponents = statusline.split(" ");
        let _version = status_compponents.next().ok_or(RequestError::BadHTTP)?;
        let _status = status_compponents.next().ok_or(RequestError::BadHTTP)?;
        let _explanation = status_compponents.next().ok_or(RequestError::BadHTTP)?;

        let mut headers = HashMap::new();
        loop {
            let header_line = lines.next().ok_or(RequestError::UnexpectedEndOfInput)?;
            if header_line.is_empty() {
                break;
            }
            let (header, value) = header_line.split_once(" ").ok_or(RequestError::BadHTTP)?;
            headers.insert(header.to_string(), value.to_string());
        }

        Ok(lines.collect::<Vec<_>>().join("\n"))
    }

    fn send_get(&self, url: &Url, stream: &mut dyn Write) -> Result<(), RequestError> {
        write!(stream, "GET {} HTTP/1.1\r\n", url.path())?;
        self.header_line(stream, "host", url.host())?;
        self.header_line(stream, "Connection", "close")?;
        self.header_line(stream, "User-Agent", "shelves")?;
        write!(stream, "\r\n")?;
        Ok(())
    }

    fn header_line(
        &self,
        stream: &mut dyn Write,
        key: &str,
        value: &str,
    ) -> Result<(), RequestError> {
        write!(stream, "{}: {}\r\n", key, value)?;
        Ok(())
    }

    /**
     * Pull the static str from cache or create and cache a new one
     */
    fn static_host(&mut self, host: &str) -> &'static str {
        match self.static_hosts.get(host) {
            Some(static_host) => *static_host,
            None => {
                let static_host = host.to_string().leak() as &'static str;
                self.static_hosts.insert(static_host);
                static_host
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
    #[error("tls error: {0}")]
    TlsError(#[from] rustls::Error),
    #[error("dns error: {0}")]
    InvalidDns(#[from] InvalidDnsNameError),
    #[error("unexpected end of input reading request")]
    UnexpectedEndOfInput,
    #[error("malformed HTTP")]
    BadHTTP,
}
