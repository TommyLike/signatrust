FROM clux/muslrust:stable as builder
ARG BINARY
LABEL Author=TommyLike<tommylikehu@gmail.com>
WORKDIR /app
COPY . /app

RUN cargo build --release --bin $BINARY --target x86_64-unknown-linux-musl

FROM alpine:latest
ARG BINARY
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/$BINARY /app
COPY ./config /app
ENTRYPOINT ["/app/$BINARY"]
