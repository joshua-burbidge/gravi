FROM rust:1.85 as builder

WORKDIR /app

COPY ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "./"]
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli

COPY .cargo ./.cargo
COPY src ./src
COPY web ./web
COPY assets ./assets

RUN cargo build --target=wasm32-unknown-unknown
RUN wasm-bindgen ./target/wasm32-unknown-unknown/debug/grav.wasm --out-dir web/generated --target web

FROM nginx:latest
COPY nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /app/web /usr/share/nginx/html
EXPOSE 80
CMD [ "nginx", "-g", "daemon off;" ]

# docker build -t gravi-rs .
# docker run -p 8080:80 gravi-rs
