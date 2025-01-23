use actix::{Actor, AsyncContext, StreamHandler};
use actix_files::NamedFile;
use actix_web::{
    http::header::{self},
    middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use actix_web_actors::ws;
use clap::Parser;
use notify::{RecursiveMode, Watcher};
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::{fs::File, io::BufReader, path::PathBuf};
use tokio::sync::broadcast;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use winapi::shared::minwindef::DWORD;
#[cfg(windows)]
use winapi::um::consoleapi::AllocConsole;
#[cfg(windows)]
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
#[cfg(windows)]
use winapi::um::processenv::GetStdHandle;
#[cfg(windows)]
use winapi::um::winbase::STD_OUTPUT_HANDLE;
#[cfg(windows)]
use winapi::um::winuser::{MessageBoxW, MB_ICONERROR, MB_OK};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to serve
    #[arg(default_value = "public")]
    path: String,

    /// Path to certifictate
    #[arg(long)]
    cert: Option<String>,

    /// Path to private key
    #[arg(long)]
    key: Option<String>,

    /// HTTP port
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Enable SSL and optionally set the port [default: 8443]
    #[arg(short, long)]
    ssl: Option<Option<u16>>,

    // Enable support for shared buffers (default: false)
    #[arg(long, default_value_t = false)]
    enable_shared_buf: bool,

    // Disable cache (Cache-Control: no-cache) (default: false)
    #[arg(long, default_value_t = false)]
    disable_cache: bool,
}
//  BOZO
struct WsSession {
    rx: broadcast::Receiver<()>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Ping(msg)) = msg {
            ctx.pong(&msg);
        }
    }

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut rx = self.rx.resubscribe();
        ctx.run_interval(std::time::Duration::from_millis(100), move |_, ctx| {
            if let Ok(_) = rx.try_recv() {
                ctx.text("reload");
            }
        });
    }
}

impl WsSession {
    fn new(rx: broadcast::Receiver<()>) -> Self {
        Self { rx }
    }
}

async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    rx: web::Data<broadcast::Sender<()>>,
) -> Result<HttpResponse> {
    let resp = ws::start(WsSession::new(rx.subscribe()), &req, stream)?;
    Ok(resp)
}

fn inject_live_reload(html: &str) -> String {
    let script = r#"
        <script>
            const ws = new WebSocket(`${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws`);
            ws.onmessage = (event) => {
                if (event.data === 'reload') window.location.reload();
            };
            ws.onclose = () => {
                console.log('LiveReload disconnected. Retrying in 1s...');
                setTimeout(() => window.location.reload(), 1000);
            };
        </script>
    "#;

    if let Some(pos) = html.rfind("</body>") {
        let mut modified = String::with_capacity(html.len() + script.len());
        modified.push_str(&html[..pos]);
        modified.push_str(script);
        modified.push_str(&html[pos..]);
        modified
    } else {
        format!("{}{}", html, script)
    }
}

struct AppData {
    reload_enabled: bool,
    enable_shared_buffer: bool,
    disable_cache: bool,
}

async fn handle_file(
    req: HttpRequest,
    path_base: web::Data<PathBuf>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse> {
    let mut filename = req.match_info().query("filename").to_string();

    let app_data = app_data.into_inner();

    if filename.is_empty() {
        filename = "index.html".to_string();
    } else if !filename.contains('.') {
        filename.push_str(".html");
    }

    let path = path_base.join(&filename);

    println!("Requested: {}", path.display());

    if path.extension().and_then(|e| e.to_str()).map_or(false, |e| e == "gz") {

        let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
        let file_contents = std::fs::read(path)?;

        let mut builder = HttpResponse::Ok();
        builder.content_type(mime_type.as_ref());
        builder.insert_header(("Content-Encoding", "gzip"));

        if filename.ends_with(".wasm.gz") {
            builder.insert_header((header::CONTENT_TYPE, "application/wasm"));
        }

        return Ok(builder.body(file_contents));
    }

    if path.extension().and_then(|e| e.to_str()).map_or(false, |e| e == "br") {

        let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
        let file_contents = std::fs::read(path)?;

        let mut builder = HttpResponse::Ok();
        builder.content_type(mime_type.as_ref());
        builder.insert_header(("Content-Encoding", "br"));

        if filename.ends_with(".wasm.br") {
            builder.insert_header((header::CONTENT_TYPE, "application/wasm"));
        }

        return Ok(builder.body(file_contents));
    }

    match NamedFile::open(&path) {
        Ok(file) => {
            if path.extension().map_or(false, |ext| ext == "html") {

                let file_content = std::fs::read_to_string(&path)?;
                let html;

                if app_data.reload_enabled  {
                    html = inject_live_reload(&file_content);
                } else { html = file_content; }

                let mut builder = HttpResponse::Ok();
                builder.content_type("text/html; charset=utf-8");
                
                if app_data.disable_cache {
                    builder.append_header(("Cache-Control", "no-cache"));
                }

                if app_data.enable_shared_buffer {
                    builder.append_header(("Cross-Origin-Embedder-Policy", "require-corp"))
                        .append_header(("Cross-Origin-Opener-Policy", "same-origin"));
                }

                return Ok(builder.body(html));
            }
            Ok(file.into_response(&req))
        }
        Err(_) => {
            let not_found_path = path_base.join("404.html");
            match NamedFile::open(not_found_path) {
                Ok(file) => Ok(file.into_response(&req)),
                Err(_) => Ok(HttpResponse::NotFound().body("404 - Not Found")),
            }
        }
    }
}

#[cfg(windows)]
fn show_error_message_box(title: &str, message: &str) {
    let title: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    let message: Vec<u16> = OsStr::new(message).encode_wide().chain(Some(0)).collect();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            MB_ICONERROR | MB_OK,
        );
    }
}

#[cfg(windows)]
fn ensure_console() {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE as DWORD);
        if handle == INVALID_HANDLE_VALUE {
            AllocConsole();
        }
    }
}

async fn error_handler(req: HttpRequest) -> HttpResponse {
    let error_message = format!(
        "Internal Server Error: Cannot process request for '{}'",
        req.uri().path()
    );

    HttpResponse::InternalServerError()
        .content_type("text/html")
        .body(format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
            <title>Server Error</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; }}
                .error {{ color: #721c24; background: #f8d7da; padding: 20px; border-radius: 15px; }}
                .footer {{ margin-top: 20px; text-align: center; font-size: 0.8em; }}
            </style>
            </head>
            <body>
            <div class="error">
                <h1>Error</h1>
                <p>{}</p>
            </div>
            <div class="footer">
                <a href="https://github.com/Pliexe/https-web-server-example/tree/main">View project on GitHub</a>
            </div>
            </body>
            </html>
            "#,
            error_message
        ))
}

fn load_ssl_config(args: &Args) -> Option<rustls::ServerConfig> {
    let cert_path = args.cert.as_deref().unwrap_or("certs/localhost.pem");
    let key_path = args.key.as_deref().unwrap_or("certs/localhost-key.pem");

    if !std::path::Path::new(cert_path).exists() || !std::path::Path::new(key_path).exists() {
        let error_msg = format!(
            "SSL certificates not found at:\n  Certificate: {}\n  Private key: {}\nRunning in HTTP-only mode.",
            cert_path, key_path
        );
        println!("{}", error_msg);
        #[cfg(windows)]
        show_error_message_box("Certificate Error", &error_msg);
        return None;
    }

    let cert_file = File::open(cert_path).map(BufReader::new);
    let key_file = File::open(key_path).map(BufReader::new);

    match (cert_file, key_file) {
        (Ok(mut cert_file), Ok(mut key_file)) => {
            let cert_chain = match rustls_pemfile::certs(&mut cert_file) {
                Ok(certs) => certs
                    .into_iter()
                    .map(Certificate)
                    .collect::<Vec<Certificate>>(),
                Err(e) => {
                    let error_msg = format!(
                        "Failed to load certificates: {}. Running in HTTP-only mode.",
                        e
                    );
                    println!("{}", error_msg);
                    #[cfg(windows)]
                    show_error_message_box("Certificate Error", &error_msg);
                    return None;
                }
            };

            let mut keys = match rustls_pemfile::pkcs8_private_keys(&mut key_file) {
                Ok(keys) => keys
                    .into_iter()
                    .map(PrivateKey)
                    .collect::<Vec<PrivateKey>>(),
                Err(e) => {
                    let error_msg = format!(
                        "Failed to load private key: {}. Running in HTTP-only mode.",
                        e
                    );
                    println!("{}", error_msg);
                    #[cfg(windows)]
                    show_error_message_box("Certificate Error", &error_msg);
                    return None;
                }
            };

            match ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(cert_chain, keys.remove(0))
            {
                Ok(config) => Some(config),
                Err(e) => {
                    let error_msg = format!(
                        "Failed to create SSL config: {}. Running in HTTP-only mode.",
                        e
                    );
                    println!("{}", error_msg);
                    #[cfg(windows)]
                    show_error_message_box("Certificate Error", &error_msg);
                    None
                }
            }
        }
        _ => {
            let error_msg = "Failed to open certificate files. Running in HTTP-only mode.";
            println!("{}", error_msg);
            #[cfg(windows)]
            show_error_message_box("Certificate Error", error_msg);
            None
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    ensure_console();

    let args = Args::parse();

    if !PathBuf::from(&args.path).exists() {
        println!(
            "Warning: Path '{}' does not exist. Creating directory.",
            args.path
        );
        std::fs::create_dir_all(&args.path)?;
    }

    let (tx, _rx) = broadcast::channel(16);
    let tx_clone = tx.clone();

    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(_) = res {
            let _ = tx_clone.send(());
        }
    })
    .unwrap();

    watcher
        .watch(
            PathBuf::from(&args.path).as_path(),
            RecursiveMode::Recursive,
        )
        .unwrap();

    let ssl_port = args.ssl.unwrap_or(Some(8443)).unwrap_or(8443);

    let ssl_config = if args.ssl.is_some() {
        load_ssl_config(&args)
    } else {
        None
    };

    if args.enable_shared_buffer == true {
        println!("Added required headers for shared buffer");
    }

    if args.disable_cache == true {
        println!("Added header for no cache");
    }

    println!("HTTP server listening on port {}", args.port);
    if ssl_config.is_some() {
        println!("HTTPS server listening on port {}", ssl_port);
        if args.cert.is_some() || args.key.is_some() {
            println!("Using custom SSL certificates:");
            println!(
                "  Certificate: {}",
                args.cert.as_deref().unwrap_or("certs/localhost.pem")
            );
            println!(
                "  Private key: {}",
                args.key.as_deref().unwrap_or("certs/localhost-key.pem")
            );
        }
    }
    println!("Serving files from: {}", args.path);

    if ssl_config.is_some() {
        let url = format!("https://localhost:{}", ssl_port);
        #[cfg(target_os = "windows")]
        std::process::Command::new("cmd")
            .args(&["/C", "start", url.as_str()])
            .spawn()
            .ok();
        #[cfg(target_os = "macos")]
        std::process::Command::new("open").arg(url).spawn().ok();
        #[cfg(target_os = "linux")]
        std::process::Command::new("xdg-open").arg(url).spawn().ok();
    } else {
        let url = format!("http://localhost:{}", args.port);
        #[cfg(target_os = "windows")]
        std::process::Command::new("cmd")
            .args(&["/C", "start", url.as_str()])
            .spawn()
            .ok();
        #[cfg(target_os = "macos")]
        std::process::Command::new("open").arg(url).spawn().ok();
        #[cfg(target_os = "linux")]
        std::process::Command::new("xdg-open").arg(url).spawn().ok();
    }

    let path_str = args.path.clone();
    let path_buf = PathBuf::from(path_str.clone());



    let app_factory = move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(path_buf.clone()))
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(AppData { reload_enabled: true, enable_shared_buffer: args.enable_shared_buffer, disable_cache: args.disable_cache }))
            .service(web::resource("/ws").route(web::get().to(ws_route)))
            .route("/", web::get().to(handle_file))
            .route("/{filename:.*}", web::get().to(handle_file))
            .service(actix_files::Files::new("/", &path_str).index_file("index.html"))
            .default_service(web::route().to(error_handler))
    };

    match ssl_config {
        Some(ssl_config) => {
            let http_server = HttpServer::new(app_factory.clone())
                .bind(format!("0.0.0.0:{}", args.port))?
                .run();

            let https_server = HttpServer::new(app_factory)
                .bind_rustls_021(format!("0.0.0.0:{}", ssl_port), ssl_config)?
                .run();

            futures_util::future::try_join(http_server, https_server).await?;
        }
        None => {
            let http_server = HttpServer::new(app_factory)
                .bind(format!("0.0.0.0:{}", args.port))?
                .run();

            http_server.await?;
        }
    }

    Ok(())
}
