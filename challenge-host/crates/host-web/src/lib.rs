use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
};

use host_protocol::{HandshakeResult, ListChallengesResult, ProgressView, WorkspaceView};
use host_runtime::{HostOverview, HostSession};
use serde::{Deserialize, Serialize};

pub use host_runtime::AdapterSpec;

type DynError = Box<dyn std::error::Error>;

pub const DEFAULT_PORT: u16 = 7878;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebOptions {
    pub adapter: AdapterSpec,
    pub port: u16,
    pub print_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootstrapResponse {
    pub handshake: HandshakeResult,
    pub challenges: ListChallengesResult,
    pub workspace: WorkspaceView,
    pub progress: ProgressView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: &'static str,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    target: String,
    body: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceRequest {
    challenge_id: String,
    #[serde(default)]
    language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SaveRequest {
    challenge_id: String,
    #[serde(default)]
    language: Option<String>,
    content: String,
}

#[derive(Debug, Deserialize)]
struct RevealHintRequest {
    challenge_id: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    test_name: Option<String>,
}

pub struct WebApp {
    asset_root: PathBuf,
    session: HostSession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrandingAsset {
    Favicon,
    MaskIcon,
}

impl WebApp {
    pub fn new(adapter: AdapterSpec) -> Result<Self, DynError> {
        Self::with_asset_root(default_asset_root(), adapter)
    }

    pub fn with_asset_root(
        asset_root: impl AsRef<Path>,
        adapter: AdapterSpec,
    ) -> Result<Self, DynError> {
        Ok(Self {
            asset_root: asset_root.as_ref().to_path_buf(),
            session: HostSession::connect_spec(&adapter)?,
        })
    }

    pub fn handle(
        &mut self,
        method: &str,
        target: &str,
        body: &[u8],
    ) -> Result<HttpResponse, DynError> {
        self.route_request(HttpRequest {
            method: method.to_string(),
            target: target.to_string(),
            body: body.to_vec(),
        })
    }

    pub fn handle_connection(&mut self, stream: &mut TcpStream) -> Result<(), DynError> {
        let request = match read_request(stream)? {
            Some(request) => request,
            None => return Ok(()),
        };

        let response = match self.route_request(request) {
            Ok(response) => response,
            Err(err) => HttpResponse {
                status: "400 Bad Request",
                content_type: "application/json; charset=utf-8",
                body: json_error(&err.to_string()),
            },
        };

        write_response(stream, response)?;
        Ok(())
    }

    fn route_request(&mut self, request: HttpRequest) -> Result<HttpResponse, DynError> {
        let (path, query) = split_target(&request.target);

        match (request.method.as_str(), path.as_str()) {
            ("GET", "/") | ("GET", "/web") | ("GET", "/web/") | ("GET", "/web/index.html") => {
                self.serve_game_asset("index.html", "text/html; charset=utf-8")
            }
            ("GET", "/styles.css") | ("GET", "/web/styles.css") => {
                self.serve_game_asset("styles.css", "text/css; charset=utf-8")
            }
            ("GET", "/app.js") | ("GET", "/web/app.js") => {
                self.serve_game_asset("app.js", "application/javascript; charset=utf-8")
            }
            ("GET", "/favicon.svg") | ("GET", "/web/favicon.svg") => {
                self.serve_branding_asset(BrandingAsset::Favicon)
            }
            ("GET", "/mask-icon.svg") | ("GET", "/web/mask-icon.svg") => {
                self.serve_branding_asset(BrandingAsset::MaskIcon)
            }
            ("GET", "/api/bootstrap") => {
                json_ok(&BootstrapResponse::from(self.session.load_overview()?))
            }
            ("GET", "/api/workspace") => {
                let challenge_id = query
                    .get("challenge_id")
                    .cloned()
                    .or_else(|| query.get("challenge").cloned())
                    .ok_or_else(|| "missing challenge_id query parameter".to_string())?;
                let language = query.get("language").cloned();
                json_ok(&self.session.load_workspace(challenge_id, language)?)
            }
            ("POST", "/api/save") => {
                let req: SaveRequest = serde_json::from_slice(&request.body)?;
                json_ok(&self.session.save_workspace(
                    req.challenge_id,
                    req.language,
                    req.content,
                )?)
            }
            ("POST", "/api/reset") => {
                let req: WorkspaceRequest = serde_json::from_slice(&request.body)?;
                json_ok(
                    &self
                        .session
                        .reset_workspace(req.challenge_id, req.language)?,
                )
            }
            ("POST", "/api/test") => {
                let req: SaveRequest = serde_json::from_slice(&request.body)?;
                json_ok(
                    &self
                        .session
                        .run_tests(req.challenge_id, req.language, req.content)?,
                )
            }
            ("POST", "/api/benchmark") => {
                let req: SaveRequest = serde_json::from_slice(&request.body)?;
                json_ok(
                    &self
                        .session
                        .benchmark(req.challenge_id, req.language, req.content)?,
                )
            }
            ("POST", "/api/reveal-hint") => {
                let req: RevealHintRequest = serde_json::from_slice(&request.body)?;
                json_ok(
                    &self
                        .session
                        .reveal_hint(req.challenge_id, req.language, req.test_name)?,
                )
            }
            _ => Ok(HttpResponse {
                status: "404 Not Found",
                content_type: "application/json; charset=utf-8",
                body: json_error("route not found"),
            }),
        }
    }

    fn serve_branding_asset(&mut self, asset: BrandingAsset) -> Result<HttpResponse, DynError> {
        let handshake = self.session.handshake()?;
        let file_name = branding_asset_file(&handshake.game_id, asset);
        serve_asset(&self.asset_root, file_name, "image/svg+xml; charset=utf-8")
    }

    fn serve_game_asset(
        &mut self,
        file_name: &str,
        content_type: &'static str,
    ) -> Result<HttpResponse, DynError> {
        let handshake = self.session.handshake()?;
        let scoped_name = game_asset_file(&handshake.game_id, file_name);
        serve_asset(&self.asset_root, &scoped_name, content_type)
    }
}

impl From<HostOverview> for BootstrapResponse {
    fn from(value: HostOverview) -> Self {
        Self {
            handshake: value.handshake,
            challenges: value.challenges,
            workspace: value.workspace,
            progress: value.progress,
        }
    }
}

pub fn run(options: WebOptions) -> Result<(), DynError> {
    let asset_root = default_asset_root();
    let mut app = WebApp::new(options.adapter)?;
    let handshake = app.session.handshake()?;
    let index_path = asset_root.join(asset_prefix(&handshake.game_id)).join("index.html");
    if !index_path.exists() {
        return Err(format!("no web UI found at {}", index_path.display()).into());
    }

    let url = web_url(options.port);
    println!("Web UI file: {}", index_path.display());
    println!("Web UI URL: {url}");

    if options.print_only {
        return Ok(());
    }

    let listener = TcpListener::bind(("127.0.0.1", options.port)).map_err(|err| {
        format!(
            "failed to bind local web server to 127.0.0.1:{}: {err}",
            options.port
        )
    })?;

    println!("Serving {url}");
    println!("Press Ctrl-C to stop.");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("warning: failed to accept connection: {err}");
                continue;
            }
        };

        if let Err(err) = app.handle_connection(&mut stream) {
            let _ = write_response(
                &mut stream,
                HttpResponse {
                    status: "500 Internal Server Error",
                    content_type: "application/json; charset=utf-8",
                    body: json_error(&err.to_string()),
                },
            );
        }
    }

    Ok(())
}

pub fn web_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}/web/index.html")
}

fn default_asset_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("web")
}

fn branding_asset_file(game_id: &str, asset: BrandingAsset) -> &'static str {
    match (game_id, asset) {
        ("fzts", BrandingAsset::Favicon) => "icons/fzts-favicon.svg",
        ("fzts", BrandingAsset::MaskIcon) => "icons/fzts-mask.svg",
        _ => match asset {
            BrandingAsset::Favicon => "icons/hazptr-favicon.svg",
            BrandingAsset::MaskIcon => "icons/hazptr-mask.svg",
        },
    }
}

fn asset_prefix(game_id: &str) -> &'static str {
    match game_id {
        "fzts" => "from-zero-to-systems",
        _ => "hazptr",
    }
}

fn game_asset_file(game_id: &str, file_name: &str) -> String {
    format!("{}/{}", asset_prefix(game_id), file_name)
}

fn serve_asset(
    asset_root: &Path,
    file_name: &str,
    content_type: &'static str,
) -> Result<HttpResponse, DynError> {
    let path = asset_root.join(file_name);
    let body =
        fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok(HttpResponse {
        status: "200 OK",
        content_type,
        body,
    })
}

fn read_request(stream: &mut TcpStream) -> Result<Option<HttpRequest>, DynError> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(None);
    }

    let request_line = request_line.trim_end_matches(['\r', '\n']);
    if request_line.is_empty() {
        return Ok(None);
    }

    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "malformed request line".to_string())?
        .to_string();
    let target = parts
        .next()
        .ok_or_else(|| "malformed request line".to_string())?
        .to_string();

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }

        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
        }
    }

    let mut body = vec![0; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    Ok(Some(HttpRequest {
        method,
        target,
        body,
    }))
}

fn write_response(stream: &mut TcpStream, response: HttpResponse) -> Result<(), DynError> {
    write!(
        stream,
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        response.content_type,
        response.body.len()
    )?;
    stream.write_all(&response.body)?;
    stream.flush()?;
    Ok(())
}

fn json_ok<T: Serialize>(value: &T) -> Result<HttpResponse, DynError> {
    Ok(HttpResponse {
        status: "200 OK",
        content_type: "application/json; charset=utf-8",
        body: serde_json::to_vec(value)?,
    })
}

fn json_error(message: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({ "error": message }))
        .unwrap_or_else(|_| b"{\"error\":\"failed to render error message\"}".to_vec())
}

fn split_target(target: &str) -> (String, HashMap<String, String>) {
    let (path, query_str) = match target.split_once('?') {
        Some((path, query_str)) => (path.to_string(), Some(query_str)),
        None => (target.to_string(), None),
    };

    let mut query = HashMap::new();
    if let Some(query_str) = query_str {
        for pair in query_str.split('&').filter(|pair| !pair.is_empty()) {
            let (key, value) = match pair.split_once('=') {
                Some((key, value)) => (key, value),
                None => (pair, ""),
            };
            query.insert(percent_decode(key), percent_decode(value));
        }
    }

    (path, query)
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut idx = 0;

    while idx < bytes.len() {
        match bytes[idx] {
            b'+' => {
                out.push(b' ');
                idx += 1;
            }
            b'%' if idx + 2 < bytes.len() => {
                let hex = &input[idx + 1..idx + 3];
                if let Ok(value) = u8::from_str_radix(hex, 16) {
                    out.push(value);
                    idx += 3;
                } else {
                    out.push(bytes[idx]);
                    idx += 1;
                }
            }
            byte => {
                out.push(byte);
                idx += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).to_string()
}
