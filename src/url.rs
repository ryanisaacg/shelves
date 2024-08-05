use std::{fmt::Display, num::ParseIntError, ops::Range};

use thiserror::Error;

#[derive(Clone)]
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
            "file" => Scheme::File,
            other => return Err(UrlError::UnknownScheme(other.to_string())),
        };

        let host_start = scheme_end + SCHEME_SEPERATOR.len();
        let host_end = url[host_start..]
            .find('/')
            .unwrap_or(url.len() - host_start)
            + host_start;

        let mut path = String::new();
        if host_end == url.len() {
            path.push('/');
        } else {
            path.push_str(&url[host_end..url.len()]);
        }

        let mut host = host_start..host_end;
        let port = match &url[host_start..host_end].find(':') {
            Some(idx) => {
                host.end = host_start + *idx;
                let port_str = &url[(host_start + idx + 1)..host_end];
                Some(port_str.parse()?)
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

    pub fn with_path(&self, path: String) -> Url {
        Url {
            path,
            ..self.clone()
        }
    }

    pub fn host(&self) -> &str {
        &self.url[self.host.clone()]
    }

    pub fn scheme(&self) -> Scheme {
        self.scheme
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.scheme.fmt(f)?;
        write!(f, "{}", self.host())?;
        if let Some(port) = self.port() {
            write!(f, ":{port}")?;
        }
        write!(f, "{}", self.path())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Scheme {
    Http,
    Https,
    File,
}

impl Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scheme::Http => write!(f, "http://"),
            Scheme::Https => write!(f, "https://"),
            Scheme::File => write!(f, "file://"),
        }
    }
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
