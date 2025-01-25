# Description
This is a small example of a HTTP/HTTPS web server in Rust with Brotli compression support. This project was created for the sole purpose of having an easier way to run a Unity exported WebGL build.

## Installation
You can get the server in two ways:
1. Download [prebuilt binaries](#binaries)
2. [Build from source](#build)

## Usage

To run the server, use the following command:
```bash
https-web-server-example [OPTIONS] [PATH]
```

### Arguments
- `PATH`: Directory to serve (default: current directory)

### Options
- `--cert <CERT>`: Path to SSL certificate (default certs/localhost.pem)
- `--key <KEY>`: Path to SSL private key (default certs/localhost-key.pem)
- `-p, --port <PORT>`: Set the port number (default: 8080)
- `-s [PORT], --ssl [PORT]`: Enable SSL and optionally set the port (default: 8443)
- `--enable-shared-buffer, --shared-buffer, --sharedbuf, --mt`: Enable support for shared buffers (default: false)
- `--disable-cache, --no-cache, --nc`: Disable cache (Cache-Control: no-cache) (default: false)
- `-h, --help`: Display help information
- `-V, --version`: Display version information

### Examples
```bash
# Serve current directory on HTTP port 8080
https-web-server

# Serve specific directory on port 3000
https-web-server -p 3000 /path/to/directory

# Enable HTTPS with self-signed certificate
https-web-server -s

# Use custom SSL certificate
https-web-server -s --cert cert.pem --key key.pem
```

## Build
To build from source, follow these steps:

1. Install [Rust](https://www.rust-lang.org/tools/install) if you haven't already
2. Clone the repository:
    ```bash
    git clone https://github.com/Pliexe/https-web-server-example.git
    ```
3. Navigate to the project directory:
    ```bash
    cd https-web-server-example
    ```
4. Build the project:
    ```bash
    cargo build --release
    ```
5. The compiled binary will be available in `target/release/`

## Binaries
Prebuilt binaries are available for:
- Windows [version 0.1.4](https://github.com/Pliexe/https-web-server-example/releases/download/0.1.4/win64.7z)
- Windows and Linux: [Download from latest release](https://github.com/Pliexe/https-web-server-example/releases/latest)

## License 
This project is licensed under the [MIT License](https://github.com/Pliexe/https-web-server-example/tree/rust/LICENSE)
