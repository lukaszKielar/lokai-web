FROM rust:1.77-slim as builder
RUN apt-get update \
    && apt-get install -y build-essential clang lld
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs npm \
    && npm install -g tailwindcss@3.4.3
WORKDIR /lokai
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /lokai
COPY --from=builder /lokai/target/release/lokai .
COPY --from=builder /lokai/static/ ./static/
COPY ./templates/ ./templates/
COPY ./migrations/ ./migrations/

ENTRYPOINT [ "/lokai/lokai" ]
