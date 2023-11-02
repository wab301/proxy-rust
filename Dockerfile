FROM rust:1.72.0-alpine as builder
LABEL maintainer="ZYQ Docker Maintainers <wab301>"

RUN echo "https://mirror.tuna.tsinghua.edu.cn/alpine/v3.4/main" >> /etc/apk/repositories
RUN apk add --update curl bash gcc musl-dev git && rm -rf /var/cache/apk/*

WORKDIR /data/server
COPY . /data/server

RUN cargo build --release

FROM alpine:3.18

COPY --from=builder /data/server/target/release/proxy-rust /data/proxy-rust/bin/
COPY --from=builder /data/server/target/release/client /data/proxy-rust/bin/
COPY --from=builder /data/server/target/release/server /data/proxy-rust/bin/

WORKDIR /data/proxy-rust

ENTRYPOINT [ "/data/proxy-rust/bin/proxy-rust" ]