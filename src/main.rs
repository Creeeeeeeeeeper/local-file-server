use clap::Parser;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use mime_guess::from_path;
use percent_encoding::percent_decode_str;
use std::{
    convert::Infallible,
    fs::OpenOptions,
    io::Write,
    net::{SocketAddr, TcpListener},
    path::PathBuf,
    sync::Arc,
};
use chrono::Local;
use tokio::fs::{self, File};
use tokio::sync::Mutex;
use tokio_util::io::ReaderStream;

type SharedLogger = Arc<Mutex<()>>;

/// 命令行参数解析结构 
#[derive(Parser, Debug)]
#[command(
    name = "file_server",
    version,
    author,
    about = "A local file server.",
    long_version = "\
file_server v1.0.0
Author: ZYG 
Email:  zyg.2005@qq.com
Repo:   https://github.com/Creeeeeeeeeeper/local-file-server
"
)]
struct Args {
    /// 起始端口号（默认 8080）
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// 根目录（默认当前目录）
    #[arg(short, long)]
    root: Option<String>,

    /// 日志模式: none / console / file / both
    #[arg(long, default_value = "none")]
    log: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let root_dir = match args.root {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().unwrap(),
    };
    let root_dir = Arc::new(root_dir);

    let log_mode = Arc::new(args.log);
    let logger = Arc::new(Mutex::new(()));

    let (addr, actual_port) = find_available_port(args.port, 20);

    println!("🚀 文件服务器已启动!");
    println!("📁 根目录: {}", root_dir.display());
    println!("🌐 地址: http://{}", addr);
    println!("🔌 端口: {}", actual_port);
    println!("📝 日志模式: {}", log_mode);
    println!();
    println!("📖 使用 file_server.exe --help 查看帮助");

    let make_service = make_service_fn(move |_conn| {
        let root_dir = root_dir.clone();
        let log_mode = log_mode.clone();
        let logger = logger.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req, root_dir.clone(), log_mode.clone(), logger.clone())
            }))
        }
    });

    if let Err(e) = Server::bind(&addr).serve(make_service).await {
        eprintln!("❌ 服务器错误: {}", e);
    }
}

fn find_available_port(start_port: u16, max_attempts: u16) -> (SocketAddr, u16) {
    for offset in 0..max_attempts {
        let port = start_port + offset;
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if TcpListener::bind(addr).is_ok() {
            return (addr, port);
        } else {
            println!("🟡 warning: 端口 {} 已被占用，尝试使用端口 {}", port, port + 1);
        }
    }

    eprintln!("❌ 没有可用端口，程序退出。");
    std::process::exit(1);
}

/// 记录请求日志
fn log_request(log_mode: &str, logger: SharedLogger, info: String) {
    let log_mode = log_mode.to_lowercase();

    if log_mode == "none" {
        return;
    }

    let time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let line = format!("[{}] {}\n", time, info);

    if log_mode == "console" || log_mode == "both" {
        print!("{}", line);
    }

    if log_mode == "file" || log_mode == "both" {
        let logger = logger.clone();
        tokio::spawn(async move {
            let _lock = logger.lock().await;
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open("access.log")
                .unwrap();
            let _ = file.write_all(line.as_bytes());
        });
    }
}

/// 处理 HTTP 请求
async fn handle_request(
    req: Request<Body>,
    root_dir: Arc<PathBuf>,
    log_mode: Arc<String>,
    logger: SharedLogger,
) -> Result<Response<Body>, Infallible> {
    let uri_path = req.uri().path();
    let decoded_path = percent_decode_str(uri_path)
        .decode_utf8()
        .unwrap_or_else(|_| uri_path.into());

    let relative_path = decoded_path.trim_start_matches('/');
    let full_path = root_dir.join(relative_path);

    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    log_request(&log_mode, logger, format!("{} {}", method, path));

    if full_path.is_dir() {
        match fs::read_dir(&full_path).await {
            Ok(mut entries) => {
                let mut html = format!(
                    "<html><head><meta charset='utf-8'><title>Index of {}</title></head><body><h3>📁 Index of {}</h3><ul>",
                    full_path.display(),
                    full_path.display()
                );

                while let Ok(Some(entry)) = entries.next_entry().await {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    let rel_link = if relative_path.is_empty() {
                        file_name_str.to_string()
                    } else {
                        format!("{}/{}", relative_path, file_name_str)
                    };

                    let file_type = entry.file_type().await.unwrap();
                    let icon = if file_type.is_dir() { "📁" } else { "📄" };

                    html += &format!("<li>{} <a href=\"/{}\">{}</a></li>", icon, rel_link, file_name_str);
                }

                html += "</ul></body></html>";

                Ok(Response::builder()
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(Body::from(html))
                    .unwrap())
            }
            Err(_) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("无法读取目录"))
                .unwrap()),
        }
    } else if full_path.is_file() {
        match File::open(&full_path).await {
            Ok(file) => {
                let mime = from_path(&full_path).first_or_octet_stream();
                let stream = ReaderStream::new(file);
                Ok(Response::builder()
                    .header("Content-Type", mime.as_ref())
                    .body(Body::wrap_stream(stream))
                    .unwrap())
            }
            Err(_) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("无法打开文件"))
                .unwrap()),
        }
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Body::from("404 - 文件未找到"))
            .unwrap())
    }
}
