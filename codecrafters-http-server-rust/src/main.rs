mod http;
mod parser;

use std::{fs, io, thread};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use clap::Parser;
use nom::AsBytes;
use crate::http::{HttpMethod, HttpRequest, HttpResponse};
use crate::parser::HttpRequestParser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory to serve GET /files/<filename> routes from
    #[arg(short, long, default_value_t = String::new())]
    directory: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let listener = TcpListener::bind("127.0.0.1:4221")?;

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                let files_dir = args.directory.clone();
                thread::spawn(|| { handle_request(files_dir, _stream) });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}

// HTTP requests have the form:
//     GET /index.html HTTP/1.1
//     Host: localhost:4221
//     User-Agent: curl/7.64.1
fn handle_request(files_dir: String, stream: TcpStream) -> io::Result<()> {
    let mut _stream = stream.try_clone().unwrap();
    let parsed_request = HttpRequestParser::from_request(BufReader::new(&_stream));

    if let Some(request) = parsed_request {
        let resp = match request.method {
            HttpMethod::GET => match request.request_target.as_str() {
                "/" => HttpResponse::new(200, HashMap::new(), None),
                "/user-agent" => response_for_user_agent_route(request),
                path if path.starts_with("/echo/") => response_for_echo_route(path.into()),
                path if path.starts_with("/files/") => serve_file_for_route(files_dir, path.into()),
                _ => HttpResponse::not_found()
            },
            HttpMethod::POST => match request.request_target.as_str() {
                path if path.starts_with("/files/") => save_file_from_route(files_dir, path.into(), request.body.into()),
                _ => HttpResponse::not_found()
            },
            _ => HttpResponse::not_found()
        };

        _stream.write_all(resp.to_string().as_bytes())?;
        _stream.flush()?;
    } else {
        _stream.write_all(HttpResponse::not_found().to_string().as_bytes())?;
        _stream.flush()?;
    }

    Ok(())
}

fn save_file_from_route(save_file_path: String, route: String, body: Option<Vec<u8>>) -> HttpResponse {
    let file_name = route.strip_prefix("/files/")
        .filter(|p| p.len() > 0);

    let Some(file_name) = file_name else {
        return HttpResponse::not_found()
    };

    let Some(body) = body else {
        return HttpResponse::not_found()
    };

    let f = File::create(save_file_path + file_name).ok();
    let file_map_success = f.map(|mut file| file.write_all(body.as_bytes()).ok()).flatten();

    if file_map_success == None {
        return HttpResponse::not_found()
    }

    HttpResponse::new(201, HashMap::new(), None)
}

fn serve_file_for_route(file_path: String, route: String) -> HttpResponse {
    let file_content = route.strip_prefix("/files/")
        .filter(|p| p.len() > 0)
        .map(|file_name| {
            // Ignore errors for now
            fs::read_to_string(file_path + file_name).ok()
        }).flatten();

    file_content.map(|content| {
        let headers = HashMap::from([
            ("Content-Type".to_string(), "application/octet-stream".to_string()),
            ("Content-Length".to_string(), content.len().to_string())]);
        HttpResponse::new(200, headers, Some(content))
    }).unwrap_or(HttpResponse::not_found())
}

fn response_for_user_agent_route(request: HttpRequest) -> HttpResponse {
    if let Some(user_agent) = request.headers.get("User-Agent") {
        let body = Some(user_agent.to_string());
        let headers = HashMap::from([
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Content-Length".to_string(), user_agent.len().to_string())]);
        HttpResponse::new(200, headers, body)
    } else {
        HttpResponse::not_found()
    }
}

fn response_for_echo_route(path: String) -> HttpResponse {
    let body = path.strip_prefix("/echo/").filter(|p| p.len() > 0);

    let mut headers: HashMap<String, String> = HashMap::new();

    if let Some(body_content) = body {
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        headers.insert("Content-Length".to_string(), body_content.len().to_string());
    }

    HttpResponse::new(200, headers, body.map(|b| b.to_owned()))
}
