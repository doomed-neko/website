use std::{
    fmt::Display,
    fs::{File, read_to_string},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::exit,
};

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = match TcpListener::bind(addr) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create listener: {e}");
            exit(1);
        }
    };

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let body = handle_connection(&mut stream);
                match body {
                    Response::String(body) => {
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                    Response::Img(bytes, kind) => {
                        let mut response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: image/{kind}\r\n\r\n",
                            bytes.len(),
                        ).as_bytes().to_vec();
                        response.extend(bytes);
                        let _ = stream.write_all(&response);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {e}");
                exit(1);
            }
        }
    }
}
enum Response {
    String(String),
    Img(Vec<u8>, ImgKind),
}
enum ImgKind {
    Jpeg,
    Png,
}
impl Display for ImgKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImgKind::Jpeg => write!(f, "jpeg"),
            ImgKind::Png => write!(f, "png"),
        }
    }
}

fn handle_connection(stream: &mut TcpStream) -> Response {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer);
    let status_line = String::from_utf8_lossy(&buffer).to_string();
    let status_line = status_line
        .lines()
        .next()
        .map(|x| x.split(" "))
        .map(|x| x.collect::<Vec<&str>>())
        .unwrap_or_default();
    if status_line.len() != 3 {
        return Response::String("you did something bad".to_string());
    }
    if status_line[0] != "GET" {
        return Response::String("tf u doin here boy".to_string());
    }
    let path = status_line[1];
    match path {
        "/" | "/index" | "/index.html" | "/index.php" | "/index.asp" => {
            Response::String(read_to_string("./src/html/index.html").unwrap())
        }
        x if x.starts_with("/imgs/") => {
            let id = x.strip_prefix("/imgs/").unwrap();
            let (path, kind) = match id {
                "1" => ("./src/html/img.jpg", ImgKind::Jpeg),
                "gol.png" => ("./src/html/gol.png", ImgKind::Png),
                "os.png" => ("./src/html/os.png", ImgKind::Png),
                "threedee.png" => ("./src/html/threedee.png", ImgKind::Png),
                _ => ("./src/html/img.jpg", ImgKind::Jpeg),
            };
            let mut file = File::open(path).unwrap();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).ok();
            Response::Img(buf, kind)
        }
        _ => Response::String("IDK what u want".to_string()),
    }
}
