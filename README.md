# local-file-server
Launch a file server on 127.0.0.1.

You can integrate this file server into your project.

This file server has the following functions (file_Server. exe - h):

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

By using the `--log` parameter, different log modes can be used. `--log` none (also the default mode) will not output or record any logs; Using the `--log` console, access logs can be outputted from the console; 

Using the `--log` file allows for the output (append) of logs to the `access.log` file, but it will not be output in the console; Using `--log` both allows for both console output and log saving.

Use the `-V` parameter to view the version, and use the `--version` parameter to view detailed information
