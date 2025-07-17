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
    future::Future,
    pin::Pin,
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
v1.1.0

    Author: ZYG 
    Email:  zyg.2005@qq.com
    Repo:   https://github.com/Creeeeeeeeeeper/local-file-server
"
)]
struct Args {
    /// 起始端口号（默认 8080）
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// 根目录（默认当前目录） [default: current directory]
    #[arg(short, long)]
    root: Option<String>,

    /// 日志模式: none / console / file / both
    #[arg(long, default_value = "none")]
    log: String,

    /// 美化输出 （默认 false）[default: false]
    #[arg(long, default_value_t = false)]
    pretty: bool,

    /// 是否允许局域网访问（默认 false）[default: false]
    #[arg(long, default_value_t = false)]
    public: bool,

    /// Enable English output.（默认 false）[default: false]
    #[arg(long, default_value_t = false)]
    en: bool,
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

    let (addr, actual_port) = find_available_port(args.port, 20, args.public);

    if args.en {
        println!("🚀 \x1B[92mFile Server has started!\x1B[0m");
        println!("📁 Root directory: {}", root_dir.display());
        println!("🌐 Address: http://127.0.0.1:{}", actual_port);
        println!("🔌 Port: {}", actual_port);
        if *log_mode == "none".to_string() {
            println!("\x1B[2m📝 Log mode: {}\x1B[0m", log_mode);
        } else {
            println!("📝 Log mode: {}", log_mode);
        }
        if args.public {
            println!("🖥️ Allowing public access");
        } else {
            println!("🖥️ Only allowing local access");
        }
        println!();
        println!("📖 Use file_server.exe -h or --help to view help");
        println!();
    } else {
        println!("🚀 \x1B[92m文件服务器已启动!\x1B[0m");
        println!("📁 根目录: {}", root_dir.display());
        println!("🌐 地址: http://127.0.0.1:{}", actual_port);
        println!("🔌 端口: {}", actual_port);
        if *log_mode == "none".to_string() {
            println!("\x1B[2m📝 日志模式: {}\x1B[0m", log_mode);
        } else {
            println!("📝 日志模式: {}", log_mode);
        }
        if args.public {
            println!("🖥️ 允许局域网访问");
        } else {
            println!("🖥️ 仅允许本机访问");
        }
        println!();
        println!("📖 使用 file_server.exe -h 或 --help 查看帮助");
        println!();
    }

    let pretty = args.pretty;
    let make_service = make_service_fn(move |_conn| {
    let root_dir = root_dir.clone();
    let log_mode = log_mode.clone();
    let logger = logger.clone();
    async move {
        Ok::<_, Infallible>(service_fn(move |req| {
            let root_dir = root_dir.clone();
            let log_mode = log_mode.clone();
            let logger = logger.clone();

            // 👇 把两个分支都包装为 Box<dyn Future> 注意 async + if/else 中的坑
            if pretty {
                Box::pin(handle_request_pretty(req, root_dir, log_mode, logger))
                    as Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>>
            } else {
                Box::pin(handle_request(req, root_dir, log_mode, logger))
                    as Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>>
            }
        }))
    }
});

    if let Err(e) = Server::bind(&addr).serve(make_service).await {
        eprintln!("❌ \x1B[91m服务器错误: {}\x1B[0m", e);
    }
}

fn find_available_port(start_port: u16, max_attempts: u16, is_public: bool) -> (SocketAddr, u16) {
    for offset in 0..max_attempts {
        let port = start_port + offset;
        let ip = if is_public {
            [0, 0, 0, 0] // 允许局域网访问
        } else {
            [127, 0, 0, 1] // 仅允许本机访问
        };
        let addr = SocketAddr::from((ip, port));
        if TcpListener::bind(addr).is_ok() {
            return (addr, port);
        } else {
            println!(
                "\x1B[93m🟡 warning: 端口 {} 已被占用，尝试使用端口 {}\x1B[0m",
                port,
                port + 1
            );
        }
    }

    eprintln!("❌ \x1B[91m没有可用端口，程序退出。\x1B[0m");
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
async fn handle_request_pretty(
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
                let current_path = if relative_path.is_empty() {
                    "/".to_string()
                } else {
                    format!("/{}", relative_path)
                };

                let display_path = if relative_path.is_empty() {
                    root_dir.display().to_string()
                } else {
                    format!("{}/{}", root_dir.display(), relative_path)
                };

                let mut html = format!(
                    r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>📁 文件服务器 - {}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
            overflow: hidden;
        }}
        
        .header {{
            background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
            color: white;
            padding: 30px;
            text-align: center;
            position: relative;
        }}
        
        .header::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: url("data:image/svg+xml,%3Csvg width='60' height='60' viewBox='0 0 60 60' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='none' fill-rule='evenodd'%3E%3Cg fill='%23ffffff' fill-opacity='0.1'%3E%3Ccircle cx='30' cy='30' r='4'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E") repeat;
        }}
        
        .header h1 {{
            font-size: 2.5em;
            margin-bottom: 10px;
            position: relative;
            z-index: 1;
        }}
        
        .header .path {{
            font-size: 1.2em;
            opacity: 0.9;
            position: relative;
            z-index: 1;
        }}
        
        .content {{
            padding: 40px;
        }}
        
        .breadcrumb {{
            margin-bottom: 30px;
            padding: 15px 20px;
            background: #f8f9fa;
            border-radius: 10px;
            border: 1px solid #e9ecef;
        }}
        
        .breadcrumb a {{
            color: #007bff;
            text-decoration: none;
            font-weight: 500;
        }}
        
        .breadcrumb a:hover {{
            text-decoration: underline;
        }}
        
        .file-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 20px;
            margin-top: 20px;
        }}
        
        .file-item {{
            background: white;
            border: 1px solid #e9ecef;
            border-radius: 15px;
            padding: 20px;
            transition: all 0.3s ease;
            position: relative;
            overflow: hidden;
            cursor: pointer;
            display: block;
            text-decoration: none;
            color: inherit;
        }}
        
        .file-item:hover {{
            transform: translateY(-5px);
            box-shadow: 0 15px 30px rgba(0, 0, 0, 0.1);
            border-color: #007bff;
            text-decoration: none;
        }}
        
        .file-item::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 4px;
            background: linear-gradient(90deg, #007bff, #00d4ff);
            transform: scaleX(0);
            transition: transform 0.3s ease;
        }}
        
        .file-item:hover::before {{
            transform: scaleX(1);
        }}
        
        .file-icon {{
            font-size: 2.5em;
            margin-bottom: 10px;
            display: block;
        }}
        
        .file-name {{
            color: #333;
            text-decoration: none;
            font-weight: 500;
            font-size: 1.1em;
            display: block;
            word-break: break-all;
        }}
        
        .file-item:hover .file-name {{
            color: #007bff;
        }}
        
        .file-type {{
            color: #666;
            font-size: 0.9em;
            margin-top: 5px;
        }}
        
        .folder {{
            background: linear-gradient(135deg, #ffeaa7 0%, #fab1a0 100%);
        }}
        
        .file {{
            background: linear-gradient(135deg, #a8e6cf 0%, #88d8c0 100%);
        }}
        
        .empty-state {{
            text-align: center;
            padding: 60px 20px;
            color: #666;
        }}
        
        .empty-state .icon {{
            font-size: 4em;
            margin-bottom: 20px;
            opacity: 0.5;
        }}
        
        .footer {{
            background: #f8f9fa;
            padding: 20px;
            text-align: center;
            color: #666;
            font-size: 0.9em;
            border-top: 1px solid #e9ecef;
        }}
        .image-preview {{
                width: 80px;
                height: 80px;
                object-fit: scale-down;
        }}
        
        @media (max-width: 768px) {{
            .file-grid {{
                grid-template-columns: 1fr;
                gap: 15px;
            }}
            
            .header h1 {{
                font-size: 2em;
            }}
            
            .content {{
                padding: 20px;
            }}
            .image-preview {{
                width: 80px;
                height: 80px;
                object-fit: scale-down;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📁 文件服务器</h1>
            <div class="path">当前路径: {}</div>
        </div>

        <script>
                    function replaceBackslashesWithForwardSlashes(inputString) {{
    return inputString.replace(/\\/g, '/');
                }}
                document.addEventListener('DOMContentLoaded', function() {{
                    document.querySelector(".path").innerHTML = replaceBackslashesWithForwardSlashes(document.querySelector(".path").innerHTML);
                    console.log(document.querySelector(".path").innerHTML)
                }});
            </script>
        
        <div class="content">
            <div class="breadcrumb">
                🏠 <a href="/">首页</a> {} 📁 {}
            </div>
            
            <div class="file-grid">
"#,
                    current_path,
                    display_path,
                    if relative_path.is_empty() { "" } else { " / " },
                    if relative_path.is_empty() { "" } else { relative_path }
                );

                let mut file_count = 0;
                let mut dir_count = 0;
                let mut items = Vec::new();

                // 收集所有文件和目录
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    let file_type = entry.file_type().await.unwrap();
                    
                    items.push((file_name_str.to_string(), file_type.is_dir()));
                    
                    if file_type.is_dir() {
                        dir_count += 1;
                    } else {
                        file_count += 1;
                    }
                }

                // 排序：目录在前，文件在后
                items.sort_by(|a, b| {
                    match (a.1, b.1) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.0.cmp(&b.0),
                    }
                });

                // 添加返回上级目录的链接
                if !relative_path.is_empty() {
                    let parent_path = if relative_path.contains('/') {
                        let parts: Vec<&str> = relative_path.split('/').collect();
                        parts[..parts.len()-1].join("/")
                    } else {
                        "".to_string()
                    };
                    
                    html += &format!(
                        r#"
                <a href="/{}" class="file-item folder">
                    <span class="file-icon">⬆️</span>
                    <div class="file-name">.. 返回上级目录</div>
                    <div class="file-type">目录</div>
                </a>
                        "#,
                        parent_path
                    );
                }

                // 生成文件和目录列表
                for (file_name, is_dir) in items {
                    let rel_link = if relative_path.is_empty() {
                        file_name.clone()
                    } else {
                        format!("{}/{}", relative_path, file_name)
                    };

                    let (icon_html, class, type_text) = if is_dir {
                        ("<span class=\"file-icon\">📁</span>".to_string(), "folder", "目录")
                    } else {
                        // 检查是否为图片文件
                        let is_image = file_name.to_lowercase().ends_with(".jpg") || 
                                      file_name.to_lowercase().ends_with(".jpeg") || 
                                      file_name.to_lowercase().ends_with(".png") || 
                                      file_name.to_lowercase().ends_with(".gif") || 
                                      file_name.to_lowercase().ends_with(".bmp") || 
                                      file_name.to_lowercase().ends_with(".webp") || 
                                      file_name.to_lowercase().ends_with(".svg");
                        
                        if is_image {
                            // 对图片显示缩略图
                            let preview_html = format!(
                                "<img src=\"/{}\" class=\"image-preview\" alt=\"{}\" loading=\"lazy\" onerror=\"this.style.display='none'; this.nextElementSibling.style.display='block'\"><span class=\"file-icon\" style=\"display:none\">🖼️</span>",
                                rel_link, file_name
                            );
                            (preview_html, "file image-item", "图片")
                        } else {
                            // 根据文件扩展名选择不同的图标
                            let icon = if file_name.ends_with(".txt") || file_name.ends_with(".md") {
                                "📄"
                            } else if file_name.ends_with(".mp4") || file_name.ends_with(".avi") || file_name.ends_with(".mov") || file_name.ends_with(".mkv") {
                                "🎬"
                            } else if file_name.ends_with(".mp3") || file_name.ends_with(".wav") || file_name.ends_with(".flac") {
                                "🎵"
                            } else if file_name.ends_with(".pdf") {
                                "📕"
                            } else if file_name.ends_with(".zip") || file_name.ends_with(".rar") || file_name.ends_with(".7z") {
                                "📦"
                            } else if file_name.ends_with(".js") || file_name.ends_with(".html") || file_name.ends_with(".css") {
                                "💻"
                            } else if file_name.ends_with(".doc") || file_name.ends_with(".docx") {
                                "📘"
                            } else if file_name.ends_with(".xls") || file_name.ends_with(".xlsx") {
                                "📗"
                            } else if file_name.ends_with(".ppt") || file_name.ends_with(".pptx") {
                                "📙"
                            } else {
                                "📄"
                            };
                            (format!("<span class=\"file-icon\">{}</span>", icon), "file", "文件")
                        }
                    };

                    html += &format!(
                        r#"
                <a href="/{}" class="file-item {}">
                    {}
                    <div class="file-name">{}</div>
                    <div class="file-type">{}</div>
                </a>
                        "#,
                        rel_link, class, icon_html, file_name, type_text
                    );
                }

                // 如果目录为空
                if dir_count == 0 && file_count == 0 {
                    html += r#"
                <div class="empty-state">
                    <div class="icon">📭</div>
                    <h3>此目录为空</h3>
                    <p>没有找到任何文件或文件夹</p>
                </div>
                    "#;
                }

                html += &format!(
                    r#"
            </div>
        </div>
        
        <div class="footer">
            📊 统计信息: {} 个文件夹, {} 个文件 | 🚀 由 Rust 文件服务器强力驱动
        </div>
    </div>
</body>
</html>
                    "#,
                    dir_count, file_count
                );

                Ok(Response::builder()
                    .header("Content-Type", "text/html; charset=utf-8")
                    .header("Access-Control-Allow-Origin", "*")
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