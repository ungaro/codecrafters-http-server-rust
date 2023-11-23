use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::str;

const USERAGENT: &str = "/user-agent";
const ECHO: &str = "/echo";


#[derive(Parser)]
#[grammar = "grammar.pest"]
struct HttpParser;

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}


#[derive(Debug, Clone, Default)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    fn parse_headers(&mut self, pairs: Pairs<Rule>) {
        for item in pairs {
            let mut kv = item.into_inner();
            let key = kv.next().unwrap().as_str().to_string();
            let value = kv.next().unwrap().as_str().to_string();
            self.headers.insert(key, value);
        }
    }
}

impl Display for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} HTTP/{}", self.method, self.url, self.version)?;
        if !self.headers.is_empty() {
            f.write_str(" [")?;
            for (i, (k, v)) in self.headers.iter().enumerate() {
                write!(f, "{}: {}", k, v)?;
                if i != self.headers.len() - 1 {
                    f.write_str(", ")?;
                }
            }
            f.write_str("]")?
        }
        Ok(())
    }
}


fn main() {
    println!("Server is running...");

    let listener = TcpListener::bind("127.0.0.1:4221")
        .expect("Could not bind to port");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Accepted new connection");

                thread::spawn(|| {
                    handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}


#[derive(Debug)]
pub struct HttpFile {
    pub requests: Vec<HttpRequest>,
}

impl<'i> TryFrom<Pair<'i, Rule>> for HttpFile {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        let iterator = pair.into_inner();
        let mut requests = vec![];
        for item in iterator {
            match item.as_rule() {
                Rule::EOI => {
                    break;
                }
                Rule::request => {
                    requests.push(item.try_into()?);
                }
                _ => {}
            }
        }
        Ok(Self { requests })
    }
}

impl Display for HttpFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.requests.is_empty() {
            writeln!(f, "No requests found")?;
            return Ok(());
        }
        for (i, req) in self.requests.iter().enumerate() {
            write!(f, "#{}\n{}\n", i, req)?;
        }
        Ok(())
    }
}

pub fn parse(input: &str) -> Result<HttpFile, Error<Rule>> {
    let file = HttpParser::parse(Rule::file, input.trim_start())
        .expect("unable to parse")
        .next()
        .unwrap();
    HttpFile::try_from(file)
}


//TODO: Convert these functions to use PEST

fn handle_client(mut stream: TcpStream) -> Result<(), std::io::Error> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let request_text = str::from_utf8(&buffer).or(Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid UTF-8")))?;
    let parsed_request = parse_request(request_text);
    println!("Parsed Request: {:?}", parsed_request);

    let response_data = match parsed_request {
        Some(request) => generate_response(request),
        None => String::from("HTTP/1.1 400 Bad Request\r\n\r\n"),
    };

    stream.write(response_data.as_bytes())?;
    stream.flush()?;
    Ok(())
}





fn parse_request(request: &str) -> Option<RequestData> {

    if lines.is_empty() {
        return None;
    }
    if lines.len() < 3 {
        return None;
    }


    let lines: Vec<&str> = request.lines().collect();
    let splitted: Vec<_> = lines[0].split_whitespace().collect();

    println!("LINES: {:?}", lines);
    println!("REQUEST: {:?}", request);



    let first_line: Vec<&str> = lines[0].split_whitespace().collect();
    if first_line.len() < 3 {
        return None;
    }
    println!("first_line: {:?}", first_line);

    Some(RequestData {
        method: first_line[0].to_string(),
        path: first_line[1].to_string(),
        http_version: first_line[2].to_string(),
        host: lines.get(1)?.split(": ").nth(1)?.to_string(),
        user_agent: lines.get(2)?.split(": ").nth(1)?.to_string(),
    })


}

fn generate_response(request: RequestData) -> String {
    println!("REQUEST: {:?}", request);

    match request.path.as_str() {
        "/" => "HTTP/1.1 200 OK \r\n\r\n".to_string(),
        path if path.starts_with(ECHO) => {
            let body = &path[ECHO.len()..];
            format_response(200, "OK", "text/plain", body)
        },
        path if path.starts_with(USERAGENT) => {
            println!("USERAGENT: {:?}", &request.user_agent);
            println!("USERAGENT: {:?}", &request);

            format_response(200, "OK", "text/plain", &request.user_agent)
        },
        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    }
}

fn format_response(status_code: u16, status_message: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        status_message,
        content_type,
        body.len(),
        body
    )
}

#[derive(Debug)]
struct RequestData {
    method: String,
    path: String,
    http_version: String,
    host: String,
    user_agent: String,
}