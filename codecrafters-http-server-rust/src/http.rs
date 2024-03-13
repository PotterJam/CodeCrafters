use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use strum_macros::EnumString;

// https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages#http_requests
#[derive(Clone, Debug)]
pub(crate) struct HttpRequest {
    pub(crate) method: HttpMethod,
    pub(crate) request_target: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Option<Vec<u8>>,
}

// https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods
#[derive(EnumString, Copy, Clone, Debug)]
pub enum HttpMethod {
    GET,
    PUT,
    POST,
    DELETE,
}

#[derive(Clone, Debug)]
pub struct HttpStatus {
    status_code: i16,
    status_message: String,
}

impl HttpStatus {
    pub fn from_status_code(status_code: i16) -> Option<HttpStatus> {
        let status_message = match status_code {
            200 => Some("OK"),
            201 => Some("Created"),
            404 => Some("Not Found"),
            _ => None
        };

        status_message.map(|m| HttpStatus {
            status_code,
            status_message: m.to_owned(),
        })
    }
}


// https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages#http_responses
#[derive(Clone, Debug)]
pub(crate) struct HttpResponse {
    protocol_version: String,
    status: HttpStatus,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpResponse {
    pub fn new(status_code: i16, headers: HashMap<String, String>, body: Option<String>) -> Self {
        HttpResponse {
            protocol_version: String::from("HTTP/1.1"),
            status: HttpStatus::from_status_code(status_code).unwrap(),
            headers,
            body,
        }
    }
    
    pub fn not_found() -> HttpResponse {
        HttpResponse::new(404, HashMap::new(), None)
    }
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut Formatter<>) -> fmt::Result {
        write!(f, "{} {:?}", self.status_code, self.status_message)
    }
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut Formatter<>) -> fmt::Result {
        write!(f, "{} {}\r\n", self.protocol_version, self.status)?;

        for (name, value) in self.headers.iter() {
            write!(f, "{}: {}\r\n", name, value)?;
        }

        if let Some(body) = &self.body {
            write!(f, "\r\n")?;
            write!(f, "{}\r\n", body)?
        }

        write!(f, "\r\n")
    }
}