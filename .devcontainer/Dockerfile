FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install clang lld \
    && apt-get autoremove -y && apt-get clean -y

RUN rustup component add rustfmt rust-docs clippy

RUN cargo install sqlx-cli --no-default-features --features native-tls,sqlite
RUN cargo install cargo-watch

RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g tailwindcss@3.4.3

USER vscode

RUN sudo chown -R $(whoami) /usr/local/* \
    && sudo chmod -R g+rwx /usr/local/
