# local-file-server  
在 127.0.0.1 上启动一个本地文件服务器，可直接集成到你的项目中。

## 使用方法  

### V 1.1.0

```
用法: file_server.exe [OPTIONS]

选项:
  -p, --port <PORT>  起始端口号（默认 8080） [default: 8080]
  -r, --root <ROOT>  根目录（默认当前目录） [default: current directory]
      --log <LOG>    日志模式: none / console / file / both [default: none]
      --pretty       美化输出 （默认 false）[default: false]
      --public       是否允许局域网访问（默认 false）[default: false]
      --en           Enable English output.（默认 false）[default: false]
  -h, --help         Print help
  -V, --version      Print version
```

新增 `--public`、 `--pretty` 和 `--en` 参数，

- `--public`可以实现是否只允许本机访问或开启局域网访问，默认只能本机访问
- `--en`参数可以允许是否使用英文输出，默认中文输出
- `--pretty`参数可以允许是否使用美化输出，默认不美化输出

美化效果：
<img width="2472" height="1364" alt="image" src="https://github.com/user-attachments/assets/6acf7941-e132-44c8-bcd0-5885d133f684" />



### V 1.0.0

```bash
file_server.exe -h
```

```
用法: file_server.exe [选项]

选项:
  -p, --port <PORT>  端口号，默认 8080
  -r, --root <ROOT>  根目录，默认为当前目录
      --log <LOG>    日志模式: none / console / file / both，默认 none
  -h, --help         打印帮助信息
  -V, --version      打印版本号
```

例如，直接执行 `file_server.exe` 将在 **8080** 端口启动文件服务器。
或者参考以下图片：
<img width="2560" height="1525" alt="image" src="https://github.com/user-attachments/assets/434cd949-3008-45bb-8841-ea91289deb40" />


---

### 参数详解

| 参数 | 说明 |
|------|------|
| `-p` / `--port` | 指定起始端口号。如果被占用，程序会自动向后 +1 尝试下一个端口 |
| `-r` / `--root` | 指定文件服务器的根目录。若省略，则默认使用 `file_server.exe` 所在的当前目录。 |
| `--log` | 设置日志模式：<br>`none`（默认）：不输出也不记录任何日志；<br>`console`：仅控制台输出访问日志；<br>`file`：仅追加写入 `access.log` 文件；<br>`both`：同时控制台输出并写入文件。 |
| `-V` / `--version` | `-V`查看版本号，`--version`查看程序详细信息。 |

---

### 示例

```bash
# 默认参数启动
file_server.exe

# 指定端口和根目录
file_server.exe -p 9000 -r D:\Share

# 控制台输出日志
file_server.exe --log console

# 控制台 + 文件双日志
file_server.exe --log both
```

---

如有任何建议或问题，欢迎提 Issue！
<br>
<br>
<br>
<br>

# local-file-server
Launch a file server on 127.0.0.1.

You can integrate this file server into your project.

This file server has the following functions (file_Server. exe - h):

## Usage

### V 1.1.0

```
Usage: file_server.exe [OPTIONS]

Options:
  -p, --port <PORT>  port  [default: 8080]
  -r, --root <ROOT>  root path（default current dir）
      --log <LOG>    log mode: none / console / file / both [default: none]
      --pretty       beautify output [default: false]
      --public       Allow LAN access [default: false]
      --en           Enable English output.[default: false]
  -h, --help         Print help
  -V, --version      Print version
```

Add three new flags: `--public`, `--pretty`, and `--en`.

- `--public` lets you choose whether the service is reachable only from localhost (default) or from the entire LAN.
- `--en` toggles output language: English when present, Chinese (default) otherwise.
- `--pretty` enables or disables pretty-printed output; defaults to off.

Example of pretty-printed output:
<img width="2472" height="1364" alt="image" src="https://github.com/user-attachments/assets/6acf7941-e132-44c8-bcd0-5885d133f684" />


### V 1.0.0

```
Usage: file_server.exe [OPTIONS]
Options:
  -p, --port <PORT>  port  [default: 8080]
  -r, --root <ROOT>  root path（default current dir）
      --log <LOG>    log mode: none / console / file / both [default: none]
  -h, --help         Print help
  -V, --version      Print version
```

For example, entering `file_derver. exe` will directly run the file server on port 8080

# Detailed explanation of parameters

The `-p` parameter can be used to specify the starting port number

The `-r` parameter can be used to specify the access root directory of the file server. If this parameter is not specified, the root directory will be the path where file_derver.exe is located

By using the `--log` parameter, different log modes can be used. `--log none` (also the default mode) will not output or record any logs; Using the `--log console`, access logs can be outputted from the console; Using the `--log file` allows for the output (append) of logs to the `access.log` file, but it will not be output in the console; Using `--log both   allows for both console output and log saving.

Use the `-V` parameter to view the version, and use the `--version` parameter to view detailed information

<br>

**If you have any good suggestions or opinions, please feel free to issue them**
