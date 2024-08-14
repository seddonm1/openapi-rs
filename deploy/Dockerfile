FROM rust:1.78.0 as builder

# Copy the present working directory to our source directory in Docker.
# Change the current directory in Docker to our source directory.
WORKDIR /app
COPY . /app

# Build our application as a static build.
RUN cargo build --release

# Download the static build of Litestream directly into the path & make it executable.
# This is done in the builder and copied as the chmod doubles the size.
ADD https://github.com/benbjohnson/litestream/releases/download/v0.3.13/litestream-v0.3.13-linux-amd64.tar.gz /tmp/litestream.tar.gz
RUN tar -C /usr/local/bin -xzf /tmp/litestream.tar.gz

# runner
FROM gcr.io/distroless/cc

COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY --from=builder /usr/local/bin/litestream /usr/local/bin/litestream

# Notify Docker that the container wants to expose a port.
EXPOSE 8080

# Copy Litestream configuration file & startup script.
COPY deploy/litestream.yml /etc/litestream.yml
COPY deploy/run.sh /scripts/run.sh

ENTRYPOINT ["/scripts/run.sh"]