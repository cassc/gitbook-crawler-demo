A demo project to crawl a gitbook website using Playwright in Rust.

# Usage

```bash
cargo run -- -h
A simple gitbook crawler using Playwright in Rust

Usage: gitbook-downloader [OPTIONS] <URL>

Arguments:
  <URL>  The root URL to start crawling from

Options:
  -e, --executable <EXECUTABLE>  Path to the browser executable file (default: None)
  -o, --output-dir <OUTPUT_DIR>  Directory to save the output
      --headless                 Headless mode (default: true)
      --ignore-external-links    Ignore external link (default: true)
  -h, --help                     Print help

Example:
cargo run -- -e /usr/bin/chromium -o output https://docs.morpho.org/morpho/overview
```

# What's missing

- Images are not downloaded
- Links are not rewritten to point to the local files
