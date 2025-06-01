# Build Stage (DotNet)
FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:9.0 AS dotnet-builder
ARG TARGETARCH
WORKDIR /build
COPY . .
RUN dotnet restore -a $TARGETARCH
WORKDIR /build/src/Zilean.ApiService
RUN dotnet publish -c Release --no-restore -a $TARGETARCH -o /app/out
WORKDIR /build/src/Zilean.Scraper
RUN dotnet publish -c Release --no-restore -a $TARGETARCH -o /app/out

# Build Stage (Rust)
FROM --platform=$TARGETARCH rust:1.87-slim AS rust-builder
ARG TARGETOS
ARG TARGETARCH

RUN apt-get update && apt-get install -y \
  perl \
  make \
  cmake \
  pkg-config \
  curl \
  build-essential \
  protobuf-compiler \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

ENV OPENSSL_STATIC=1
ENV OPENSSL_NO_VENDOR=0

WORKDIR /build

COPY src/RustServer/ .
COPY src/Protos /Protos
COPY src/ParsettOverToRust /ParsettOverToRust

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    case "$TARGETARCH" in \
        *amd64 | */amd64/*) PLATFORM=x86_64 ;; \
        *arm64 | */arm64/* ) PLATFORM=aarch64 ;; \
        * ) echo "Unexpected TARGETARCH '$TARGETARCH'" >&2; exit 1 ;; \
    esac && \
    rustup target add $PLATFORM-unknown-linux-gnu && \
    export TARGET=$PLATFORM-unknown-linux-gnu && \
    cargo build --release --target=$TARGET --locked && \
    cp target/$TARGET/release/zilean_rust /zilean_rust

# Run Stage
FROM mcr.microsoft.com/dotnet/aspnet:9.0

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    libicu72 \
    && rm -rf /var/lib/apt/lists/*

ENV DOTNET_RUNNING_IN_CONTAINER=true
ENV DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=false
ENV ASPNETCORE_URLS=http://+:8181

WORKDIR /app
VOLUME /app/data
COPY --from=dotnet-builder /app/out .

COPY --from=rust-builder --chmod=0775 /zilean_rust /app/zilean_rust

ENTRYPOINT ["./zilean-api"]
