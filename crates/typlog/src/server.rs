use std::fs;
use std::io::Read;
use std::path::Path;

use anyhow::{Result, anyhow, bail};
use tiny_http::{Header, Response, Server, StatusCode};

pub fn serve_public(port: u16) -> Result<()> {
    let root = Path::new("public");
    if !root.exists() {
        bail!("缺少 public/ 目录，请先执行 generate");
    }

    let address = format!("127.0.0.1:{port}");
    let server = Server::http(&address).map_err(|e| anyhow!("无法监听地址 {address}: {e}"))?;
    println!("本地预览已启动: http://{address}");

    for request in server.incoming_requests() {
        let request_url = request.url().to_string();
        let url_path = request_url.split('?').next().unwrap_or("/");
        let rel = url_path.trim_start_matches('/');
        let mut fs_path = if rel.is_empty() {
            root.join("index.html")
        } else {
            root.join(rel)
        };

        if fs_path.is_dir() {
            fs_path = fs_path.join("index.html");
        }
        if !fs_path.exists() && rel.is_empty() {
            fs_path = root.join("posts");
        }
        if fs_path.is_dir() {
            fs_path = fs_path.join("index.html");
        }

        let response = match fs::File::open(&fs_path) {
            Ok(mut file) => {
                let mut body = Vec::new();
                file.read_to_end(&mut body)?;
                let mut resp = Response::from_data(body);
                if let Ok(header) = Header::from_bytes(
                    b"Content-Type".as_slice(),
                    guess_content_type(&fs_path).as_bytes(),
                ) {
                    resp = resp.with_header(header);
                }
                resp
            }
            Err(_) => Response::from_string("Not Found").with_status_code(StatusCode(404)),
        };
        let _ = request.respond(response);
    }
    Ok(())
}

fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::guess_content_type;

    #[test]
    fn content_type_should_detect_html() {
        assert_eq!(
            guess_content_type(Path::new("a/b/c.html")),
            "text/html; charset=utf-8"
        );
    }
}
