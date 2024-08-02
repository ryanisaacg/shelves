use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::TcpStream,
    ops::Range,
};

use thiserror::Error;

pub struct Url {
    url: String,
    host: Range<usize>,
    path: String,
}

impl Url {
    pub fn new(url: String) -> Url {
        let split = url.find('/').unwrap_or(url.len());
        let path = format!("/{}", &url[split..url.len()]);

        Url {
            url,
            host: 0..split,
            path,
        }
    }

    fn host(&self) -> &str {
        &self.url[self.host.clone()]
    }

    pub fn request(&self) -> Result<String, RequestError> {
        let host = self.host();
        let mut stream = TcpStream::connect((host, 80))?;
        /*
        request = "GET {} HTTP/1.0\r\n".format(self.path)
               request += "Host: {}\r\n".format(self.host)
               request += "\r\n"
                */
        write!(stream, "GET {} HTTP/1.0\r\n", self.path.as_str())?;
        write!(stream, "Host: {}\r\n", host)?;
        write!(stream, "\r\n")?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;

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
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
    #[error("unexpected end of input reading request")]
    UnexpectedEndOfInput,
    #[error("malformed HTTP")]
    BadHTTP,
}
