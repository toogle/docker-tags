# Docker Tags

CLI tool to list tags for Docker images with sensible sorting.

## Features
- Works with Docker Hub and other registries, compliant with OCI Distribution Specification.
- Sorts semantic versions with newest first and falls back to alphabetical for non-semver tags.
- Supports authentication via Docker credentials (`~/.docker/config.json`).

## Basic Usage (CLI)
- `docker-tags [<registry>/][<namespace>/]<image>` — list tags for an image (e.g., `docker-tags alpine`).
- `docker-tags -r [<registry>/][<namespace>/]<image>` — same, but reverse the order (e.g., `docker-tags -r quay.io/prometheus/prometheus`).

## Build from Source
1) Ensure the Rust toolchain is installed (via `rustup`).
2) Build the binary: `cargo build --release`.
3) The compiled executable will be at `target/release/docker-tags`.

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
