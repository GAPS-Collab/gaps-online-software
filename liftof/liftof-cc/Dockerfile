FROM ghcr.io/cross-rs/x86_64-unknown-linux-musl

# Install your custom package
RUN apt-get update && apt-get install -y \
    lm-sensors  \
&& rm -rf /var/lib/apt/lists/*
