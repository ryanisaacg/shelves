use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::TcpStream,
    num::ParseIntError,
    ops::Range,
    sync::Arc,
};

use thiserror::Error;

pub struct Url {
    url: String,
    scheme: Scheme,
    host: Range<usize>,
    path: String,
    port: Option<u16>,
}

const SCHEME_SEPERATOR: &str = "://";

impl Url {
    pub fn new(url: String) -> Result<Url, UrlError> {
        let scheme_end = url
            .find(SCHEME_SEPERATOR)
            .ok_or(UrlError::NoSchemeProvided)?;
        let scheme = match &url[..scheme_end] {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            other => return Err(UrlError::UnknownScheme(other.to_string())),
        };

        let host_start = scheme_end + SCHEME_SEPERATOR.len();
        let host_end = url[host_start..]
            .find('/')
            .unwrap_or(url.len() - host_start)
            + host_start;
        let path = format!("/{}", &url[host_end..url.len()]);

        let mut host = host_start..host_end;
        let port = match &url[host_start..host_end].find(':') {
            Some(idx) => {
                host.end = host_start + *idx;
                let port_str = &url[(host_start + idx + 1)..(host_start + host_end)];
                Some(dbg!(port_str).parse()?)
            }
            None => None,
        };

        Ok(Url {
            url,
            scheme,
            port,
            host,
            path,
        })
    }

    fn host(&self) -> &str {
        dbg!(&self.url[self.host.clone()])
    }

    pub fn request(&self) -> Result<String, RequestError> {
        let host = self.host();

        let mut response = String::new();
        match self.scheme {
            Scheme::Http => {
                let mut stream = TcpStream::connect((host, self.port.unwrap_or(80)))?;

                self.send_get(&mut stream)?;
                stream.read_to_string(&mut response)?;
            }
            Scheme::Https => {
                // Set up TLS connection
                let root_store = rustls::RootCertStore::from_iter(
                    webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
                );
                // TODO: refactor ClientConfig out of URL.request
                let config = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                let rc_config = Arc::new(config);
                // TODO: if we must leak, at least cache the leaked memory
                let static_host = host.to_string().leak() as &'static str;
                let example_com = static_host.try_into().unwrap();
                let mut client = rustls::ClientConnection::new(rc_config, example_com)?;

                let mut stream = TcpStream::connect((host, self.port.unwrap_or(443)))?;
                let mut stream = rustls::Stream::new(&mut client, &mut stream);

                self.send_get(&mut stream)?;
                stream.read_to_string(&mut response)?;
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

    fn send_get(&self, stream: &mut dyn Write) -> Result<(), RequestError> {
        write!(stream, "GET {} HTTP/1.0\r\n", self.path.as_str())?;
        write!(stream, "Host: {}\r\n", self.host())?;
        write!(stream, "\r\n")?;
        Ok(())
    }
}

enum Scheme {
    Http,
    Https,
}

#[derive(Debug, Error)]
pub enum UrlError {
    #[error("invalid URL: must have a scheme")]
    NoSchemeProvided,
    #[error("invalid port integer: {0}")]
    InvalidPortInt(#[from] ParseIntError),
    #[error("unknown scheme: {0}")]
    UnknownScheme(String),
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
    #[error("tls error: {0}")]
    TlsError(#[from] rustls::Error),
    #[error("unexpected end of input reading request")]
    UnexpectedEndOfInput,
    #[error("malformed HTTP")]
    BadHTTP,
}
