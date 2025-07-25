FROM rust:1.85 as builder

WORKDIR /app

COPY ["Cargo.toml", "Cargo.lock", "./"]
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli

COPY benches ./benches
COPY src ./src
COPY web ./web
COPY assets ./assets

RUN cargo build --target=wasm32-unknown-unknown --release
RUN wasm-bindgen ./target/wasm32-unknown-unknown/release/grav.wasm --out-dir web/generated --target web

FROM nginx:latest
COPY nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /app/web /usr/share/nginx/html
EXPOSE 80
CMD [ "nginx", "-g", "daemon off;" ]

# docker build -t gravi-rs .
# docker run --rm -p 80:80 gravi-rs
