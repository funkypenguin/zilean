# Build Stage (DotNet)
FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:9.0-alpine AS dotnet-builder
ARG TARGETARCH
ENV PROTOBUF_PROTOC=/usr/bin/protoc
ENV gRPC_PluginFullPath=/usr/bin/grpc_csharp_plugin
RUN apk add protobuf protobuf-dev grpc grpc-plugins
WORKDIR /build
COPY . .
RUN dotnet restore -a $TARGETARCH
WORKDIR /build/src/Zilean.ApiService
RUN dotnet publish -c Release --no-restore -a $TARGETARCH -o /app/out
WORKDIR /build/src/Zilean.Scraper
RUN dotnet publish -c Release --no-restore -a $TARGETARCH -o /app/out

# Build Stage (Rust)
FROM --platform=$TARGETARCH rust:1.87-alpine AS rust-builder
ARG TARGETOS
ARG TARGETARCH
RUN apk add --no-cache musl-dev pkgconfig perl make protobuf-dev
WORKDIR /build

COPY src/RustServer/ ./
COPY src/Protos /Protos
COPY src/ParsettOverToRust /ParsettOverToRust

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    case "$TARGETARCH" in \
        *amd64 | */amd64/*) PLATFORM=x86_64 ;; \
        *arm64 | */arm64/* ) PLATFORM=aarch64 ;; \
        * ) echo "Unexpected TARGETARCH '$TARGETARCH'" >&2; exit 1 ;; \
    esac; \
    rustup target add $PLATFORM-unknown-linux-musl; \
    cargo build --release --target=$PLATFORM-unknown-linux-musl; \
    cp /build/target/$PLATFORM-unknown-linux-musl/release/zilean_rust /zilean_rust

# Run Stage
FROM mcr.microsoft.com/dotnet/aspnet:9.0-alpine

RUN echo "https://dl-cdn.alpinelinux.org/alpine/v3.18/main" > /etc/apk/repositories && \
    echo "https://dl-cdn.alpinelinux.org/alpine/v3.18/community" >> /etc/apk/repositories && \
    apk update

RUN apk add --update --no-cache \
    curl \
    icu-libs

ENV DOTNET_RUNNING_IN_CONTAINER=true
ENV DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=false
ENV ASPNETCORE_URLS=http://+:8181

WORKDIR /app
VOLUME /app/data
COPY --from=dotnet-builder /app/out .

COPY --from=rust-builder --chmod=0775 /zilean_rust /app/zilean_rust

ENTRYPOINT ["./zilean-api"]
