// ===== AFRI-HTTP — Notre propre serveur HTTP from scratch =====
// 100% souverain — zéro dépendance externe
// Utilise std::net::TcpListener pour écouter les connexions
// Parse les requêtes HTTP/1.1 et génère les réponses

use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::thread;

// ===== Request =====
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub peer_addr: String,
}

impl HttpRequest {
    pub fn query_str(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(&key.to_lowercase()).map(|s| s.as_str())
    }

    pub fn body_str(&self) -> &str {
        std::str::from_utf8(&self.body).unwrap_or("")
    }

    pub fn form(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let body = self.body_str();
        for pair in body.split('&') {
            if let Some(eq) = pair.find('=') {
                let key = url_decode(&pair[..eq]);
                let val = url_decode(&pair[eq+1..]);
                map.insert(key, val);
            }
        }
        map
    }

    pub fn cookie(&self, name: &str) -> Option<String> {
        let cookie_header = self.header("cookie")?;
        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if let Some(eq) = cookie.find('=') {
                let key = &cookie[..eq];
                let val = &cookie[eq+1..];
                if key == name {
                    return Some(val.to_string());
                }
            }
        }
        None
    }
}

// ===== Response =====
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn ok(body: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        HttpResponse {
            status: 200,
            headers,
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn ok_bytes(body: Vec<u8>, content_type: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        HttpResponse {
            status: 200,
            headers,
            body,
        }
    }

    pub fn json(body: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        HttpResponse {
            status: 200,
            headers,
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn redirect(path: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Location".to_string(), path.to_string());
        HttpResponse {
            status: 302,
            headers,
            body: Vec::new(),
        }
    }

    pub fn forbidden(body: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        HttpResponse {
            status: 403,
            headers,
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn not_found() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        HttpResponse {
            status: 404,
            headers,
            body: b"<h1>404 - Page introuvable</h1>".to_vec(),
        }
    }

    pub fn content_type(mut self, ct: &str) -> Self {
        self.headers.insert("Content-Type".to_string(), ct.to_string());
        self
    }

    pub fn header(mut self, key: &str, val: &str) -> Self {
        self.headers.insert(key.to_string(), val.to_string());
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = body.as_bytes().to_vec();
        self
    }

    pub fn body_bytes(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn set_cookie(mut self, name: &str, value: &str, max_age_secs: i64) -> Self {
        let cookie = if max_age_secs <= 0 {
            format!("{}=; Path=/; Max-Age=0", name)
        } else {
            format!("{}={}; Path=/; Max-Age={}", name, value, max_age_secs)
        };
        self.headers.insert("Set-Cookie".to_string(), cookie);
        self
    }

    pub fn append_header(mut self, key: &str, val: &str) -> Self {
        // For Location redirects, we use the key directly
        self.headers.insert(key.to_string(), val.to_string());
        self
    }

    fn to_bytes(&self) -> Vec<u8> {
        let status_text = match self.status {
            200 => "OK",
            302 => "Found",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "OK",
        };

        let mut out = format!("HTTP/1.1 {} {}\r\n", self.status, status_text);
        for (k, v) in &self.headers {
            out.push_str(&format!("{}: {}\r\n", k, v));
        }
        out.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        out.push_str("Connection: close\r\n");
        out.push_str("\r\n");

        let mut bytes = out.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

// ===== URL Decode =====
fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => { result.push(' '); i += 1; }
            b'%' if i + 2 < bytes.len() => {
                let hex = &s[i+1..i+3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    // Check for multi-byte UTF-8
                    if byte < 0x80 {
                        result.push(byte as char);
                        i += 3;
                    } else {
                        // Collect multi-byte sequence
                        let mut utf8_bytes = vec![byte];
                        let extra = if byte >= 0xF0 { 3 } else if byte >= 0xE0 { 2 } else { 1 };
                        i += 3;
                        for _ in 0..extra {
                            if i + 2 < bytes.len() && bytes[i] == b'%' {
                                let h = &s[i+1..i+3];
                                if let Ok(b) = u8::from_str_radix(h, 16) {
                                    utf8_bytes.push(b);
                                    i += 3;
                                } else { break; }
                            } else { break; }
                        }
                        if let Ok(st) = std::str::from_utf8(&utf8_bytes) {
                            result.push_str(st);
                        }
                    }
                } else {
                    result.push(bytes[i] as char);
                    i += 1;
                }
            }
            c => { result.push(c as char); i += 1; }
        }
    }
    result
}

// ===== Parse query string =====
fn parse_query(qs: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in qs.split('&') {
        if let Some(eq) = pair.find('=') {
            let key = url_decode(&pair[..eq]);
            let val = url_decode(&pair[eq+1..]);
            map.insert(key, val);
        } else if !pair.is_empty() {
            map.insert(url_decode(pair), String::new());
        }
    }
    map
}

// ===== Parse HTTP request from stream =====
fn parse_request(stream: &mut TcpStream) -> Option<HttpRequest> {
    let peer_addr = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let mut reader = BufReader::new(stream);
    
    // Read request line
    let mut line = String::new();
    if reader.read_line(&mut line).ok()? == 0 {
        return None;
    }
    
    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    
    let method = parts[0].to_string();
    let raw_path = parts[1].to_string();
    
    // Split path and query
    let (path, query) = if let Some(q) = raw_path.find('?') {
        let p = raw_path[..q].to_string();
        let qs = &raw_path[q+1..];
        (p, parse_query(qs))
    } else {
        (raw_path, HashMap::new())
    };
    
    // Read headers
    let mut headers = HashMap::new();
    loop {
        let mut header_line = String::new();
        let n = reader.read_line(&mut header_line).ok()?;
        if n == 0 || header_line.trim().is_empty() {
            break;
        }
        if let Some(colon) = header_line.find(':') {
            let key = header_line[..colon].trim().to_lowercase();
            let val = header_line[colon+1..].trim().to_string();
            headers.insert(key, val);
        }
    }
    
    // Read body if Content-Length present
    let mut body = Vec::new();
    if let Some(cl) = headers.get("content-length") {
        if let Ok(len) = cl.parse::<usize>() {
            if len > 0 {
                body.resize(len, 0);
                reader.read_exact(&mut body).ok()?;
            }
        }
    }
    
    Some(HttpRequest {
        method,
        path,
        query,
        headers,
        body,
        peer_addr,
    })
}

// ===== Server =====
pub fn serve<F>(addr: &str, handler: F)
where
    F: Fn(HttpRequest) -> HttpResponse + Send + Sync + 'static,
{
    let listener = TcpListener::bind(addr).unwrap_or_else(|e| {
        eprintln!("Erreur bind {}: {}", addr, e);
        std::process::exit(1);
    });
    
    println!("🌐 Serveur HTTP souverain sur {}", addr);
    
    let handler = std::sync::Arc::new(handler);
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let h = handler.clone();
                thread::spawn(move || {
                    if let Some(req) = parse_request(&mut stream) {
                        let resp = h(req);
                        let _ = stream.write_all(&resp.to_bytes());
                        let _ = stream.flush();
                    }
                });
            }
            Err(e) => {
                eprintln!("Connexion erreur: {}", e);
            }
        }
    }
}
