# Base image to build from. We use this over the official rust image so that 
# we can the rustup musl target in order to create a stand-alone exe.
# We use a multi-stage build to ensure that the final image only contains the binary we need
# and not external/project deps. This helps in improving the final image size.
FROM ekidd/rust-musl-builder:stable as builder

LABEL version="0.1.0"
LABEL author="jaiswalsanskar078@gmail.com"

# Create dummy project and install deps. This is a hacky way of caching deps to save build time.
RUN USER=root cargo new kv-store
WORKDIR /home/rust/src/kv-store
COPY Cargo.toml Cargo.lock ./
RUN cargo install --target x86_64-unknown-linux-musl --path .
RUN rm src/*.rs

# Copy source code and compile.
ADD --chown=rust:rust . ./
RUN cargo install --target x86_64-unknown-linux-musl --bin server --path .

# Second phase of the multi-stage build.
FROM alpine:latest
RUN apk update && apk add --no-cache ca-certificates

# Transfer the binary from the builder container to this one.
COPY --from=builder /home/rust/src/kv-store/target/x86_64-unknown-linux-musl/release/server .

CMD ["./server"]
