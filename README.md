# Description
This is a small example of a HTTP/HTTPS web server in Rust with Brotli compression support. This project was created for the sole purpose of having an easier way to run a Unity exported WebGL build.

## Installation
You can get the server in two ways:
1. Download [prebuilt binaries](#binaries)
2. [Build from source](#build)

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
- Windows [version 0.1.0](https://github.com/Pliexe/https-web-server-example/releases/tag/0.1.0/win64.7z)
- Windows and Linux: [Download from latest release](https://github.com/Pliexe/https-web-server-example/releases/latest)

## License 
This project is licensed under the [MIT License](https://github.com/Pliexe/https-web-server-example/tree/rust/LICENSE)
