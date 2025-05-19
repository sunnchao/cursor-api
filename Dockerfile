ARG TARGETARCH
FROM --platform=linux/${TARGETARCH} rustlang/rust:nightly-bookworm-slim as builder

ARG TARGETARCH

WORKDIR /app
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential protobuf-compiler pkg-config libssl-dev nodejs npm openssl \
    && rm -rf /var/lib/apt/lists/*

COPY . .
RUN case "$TARGETARCH" in amd64) TARGET_CPU="x86-64-v2" ;; arm64) TARGET_CPU="neoverse-n1" ;; *) echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; esac && RUSTFLAGS="-C link-arg=-s -C target-cpu=$TARGET_CPU" cargo +nightly build --release && cp target/release/cursor-api /app/cursor-api

# 运行阶段
ARG TARGETARCH
FROM --platform=linux/${TARGETARCH} debian:bookworm-slim

WORKDIR /app
ENV TZ=Asia/Shanghai

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates tzdata openssl \
    && rm -rf /var/lib/apt/lists/* && \
    groupadd -r cursorapi && useradd -r -g cursorapi cursorapi

COPY --from=builder /app/cursor-api .
RUN chown -R cursorapi:cursorapi /app

ENV PORT=3000
EXPOSE ${PORT}

USER cursorapi
CMD ["./cursor-api"]