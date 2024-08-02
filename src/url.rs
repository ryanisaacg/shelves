use std::{num::ParseIntError, ops::Range};

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

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Scheme {
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
