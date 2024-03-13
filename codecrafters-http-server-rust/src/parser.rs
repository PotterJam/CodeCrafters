use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::str::FromStr;
use crate::http::{HttpMethod, HttpRequest};

pub struct HttpRequestParser();

impl HttpRequestParser {
    // TODO: needs a bunch of error handling to a common result type, lots of unwrap for now.
    pub(crate) fn from_request(mut reader: BufReader<&TcpStream>) -> Option<HttpRequest> {

        // I hate this but the ownership of the buffered line reader changing to an exact byte reader
        // made me want to chop my hands off and feed it to some dogs
        let mut first_line = String::new();
        reader.read_line(&mut first_line).unwrap();

        let first_line_parts: Vec<&str> = first_line.split_whitespace().collect();

        let [http_method, path, ..] = first_line_parts[..] else {
            return None;
        };

        let method = HttpMethod::from_str(http_method).unwrap();

        let mut headers: HashMap<String, String> = HashMap::new();

        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            // So we can check for content length and get the body if we need to
            if line == "\r\n" { break; }

            let (key, value) = line.split_once(": ").unwrap();
            headers.insert(key.to_string(), value.trim().to_string());
        }

        let mut body: Option<Vec<u8>> = None;
        if let Some(content_length) = headers.get("Content-Length") {
            let len = content_length.parse::<usize>().unwrap();
            let mut buf = vec![0u8; len];
            body = reader.read_exact(&mut buf).ok().map(|_| buf);
        }

        Some(HttpRequest {
            request_target: path.to_string(),
            method,
            headers,
            body,
        })
    }
}