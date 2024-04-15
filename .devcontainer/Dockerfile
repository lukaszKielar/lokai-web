FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install clang lld \
   && apt-get autoremove -y && apt-get clean -y

RUN rustup toolchain install nightly \
   && rustup default nightly \
   && rustup target add wasm32-unknown-unknown \
   && rustup component add rustfmt rust-docs clippy

RUN cargo install cargo-generate
RUN cargo install cargo-leptos
RUN cargo install leptosfmt
RUN cargo install trunk --no-default-features --features native-tls
RUN cargo install sqlx-cli --no-default-features --features native-tls,sqlite

RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
   && apt-get install -y nodejs

RUN npm install sqlite3@5.1.1

USER vscode

RUN sudo chown -R $(whoami) /usr/local/* \
    && sudo chmod -R g+rwx /usr/local/
