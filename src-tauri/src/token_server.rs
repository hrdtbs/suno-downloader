use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};

use serde::Deserialize;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};
use uuid::Uuid;

use crate::config::paths::{token_server_url, TOKEN_SERVER_HOST, TOKEN_SERVER_PORT};
use crate::config::session::{normalize_token, save_session};
use crate::suno::auth::verify_session;
use crate::suno::types::TokenServerStatus;

const SERVICE_NAME: &str = "suno-downloader";

#[derive(Debug, Deserialize)]
struct TokenBody {
    token: Option<String>,
    jwt: Option<String>,
    #[serde(rename = "deviceId")]
    device_id: Option<String>,
    #[serde(rename = "device_id")]
    device_id_snake: Option<String>,
}

pub struct TokenServerManager {
    handle: Mutex<Option<JoinHandle<()>>>,
    device_id: Arc<Mutex<String>>,
    block_extension_auth: Arc<AtomicBool>,
}

impl TokenServerManager {
    pub fn new(block_extension_auth: Arc<AtomicBool>) -> Self {
        Self {
            handle: Mutex::new(None),
            device_id: Arc::new(Mutex::new(Uuid::new_v4().to_string())),
            block_extension_auth,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        {
            let guard = self
                .handle
                .lock()
                .map_err(|_| anyhow::anyhow!("token server lock poisoned"))?;
            if guard.is_some() {
                return Ok(());
            }
        }

        if let Ok(response) = reqwest::Client::new()
            .get(format!("{}/status", token_server_url()))
            .timeout(std::time::Duration::from_millis(1500))
            .send()
            .await
        {
            if response.status().is_success() {
                return Ok(());
            }
        }

        let mut guard = self
            .handle
            .lock()
            .map_err(|_| anyhow::anyhow!("token server lock poisoned"))?;
        if guard.is_some() {
            return Ok(());
        }

        let device_id = Arc::clone(&self.device_id);
        let block_extension_auth = Arc::clone(&self.block_extension_auth);
        let handle = thread::spawn(move || run_token_server(&device_id, &block_extension_auth));
        *guard = Some(handle);
        Ok(())
    }

    pub async fn status() -> TokenServerStatus {
        let running = match reqwest::Client::new()
            .get(format!("{}/status", token_server_url()))
            .timeout(std::time::Duration::from_millis(1500))
            .send()
            .await
        {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(body) => body
                    .get("service")
                    .and_then(|value| value.as_str())
                    .map(|service| {
                        service == SERVICE_NAME || service == "suno" || service == "suno-sync-mini"
                    })
                    .unwrap_or(false),
                Err(_) => false,
            },
            Err(_) => false,
        };

        TokenServerStatus {
            running,
            url: token_server_url(),
            port: TOKEN_SERVER_PORT,
        }
    }
}

fn run_token_server(device_id: &Arc<Mutex<String>>, block_extension_auth: &Arc<AtomicBool>) {
    let address = format!("{TOKEN_SERVER_HOST}:{TOKEN_SERVER_PORT}");
    let server = match Server::http(&address) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("Failed to start token server: {error}");
            return;
        }
    };

    for request in server.incoming_requests() {
        if let Err(error) = handle_request(request, device_id, block_extension_auth) {
            eprintln!("Token server error: {error}");
        }
    }
}

fn handle_request(
    mut request: Request,
    device_id: &Arc<Mutex<String>>,
    block_extension_auth: &Arc<AtomicBool>,
) -> anyhow::Result<()> {
    if request.method() == &Method::Options {
        let response = json_response(StatusCode(204), "");
        let _ = request.respond(response);
        return Ok(());
    }

    let path = request.url().split('?').next().unwrap_or("");

    if request.method() == &Method::Get && path == "/status" {
        let body = serde_json::json!({ "ok": true, "service": SERVICE_NAME }).to_string();
        let response = json_response(StatusCode(200), &body);
        let _ = request.respond(response);
        return Ok(());
    }

    if request.method() == &Method::Post && path == "/token" {
        if block_extension_auth.load(Ordering::Relaxed) {
            let response = json_response(
                StatusCode(403),
                r#"{"ok":false,"error":"extension auth blocked"}"#,
            );
            let _ = request.respond(response);
            return Ok(());
        }

        let mut body = String::new();
        request.as_reader().read_to_string(&mut body)?;

        let parsed: TokenBody = serde_json::from_str(&body)?;
        let token = parsed
            .token
            .or(parsed.jwt)
            .ok_or_else(|| anyhow::anyhow!("token is required"))?;

        let jwt = normalize_token(&token);
        let resolved_device_id = parsed
            .device_id
            .or(parsed.device_id_snake)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| {
                device_id
                    .lock()
                    .map_or_else(|_| Uuid::new_v4().to_string(), |guard| guard.clone())
            });

        if let Ok(mut guard) = device_id.lock() {
            guard.clone_from(&resolved_device_id);
        }

        token_server_runtime().block_on(async {
            verify_session(&jwt, &resolved_device_id).await?;
            save_session(&jwt, &resolved_device_id).await
        })?;

        let response = json_response(StatusCode(200), r#"{"ok":true}"#);
        let _ = request.respond(response);
        return Ok(());
    }

    let response = json_response(StatusCode(404), r#"{"ok":false,"error":"not found"}"#);
    let _ = request.respond(response);
    Ok(())
}

fn token_server_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build token server runtime")
    })
}

fn json_response(status: StatusCode, body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(Header::from_bytes("Content-Type", "application/json").unwrap())
        .with_header(Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap())
        .with_header(
            Header::from_bytes("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap(),
        )
        .with_header(Header::from_bytes("Access-Control-Allow-Headers", "Content-Type").unwrap())
}
