# Build stage
FROM rust:bookworm AS builder
 
WORKDIR /app
COPY . .
RUN cargo build --release
 
# Final run stage
FROM debian:bookworm-slim AS runner

RUN apt-get update -y && apt-get install -y libssl3 ca-certificates libglib2.0-0
WORKDIR /app
COPY --from=builder /app/target/release/l4l /app/l4l
CMD ["/app/l4l"]