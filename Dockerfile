# Build stage
FROM rust:bookworm AS builder
 
WORKDIR /app
COPY . .
RUN cargo build --release
 
# Final run stage
FROM debian:bookworm-slim AS runner

RUN apt-get update -y && apt-get install -y libssl3 ca-certificates libglib2.0-0 libnss3-dev libxss1 libatk1.0-0 libatk-bridge2.0-0 libcups2 libxcomposite1 libxrandr2 libxdamage1 libpango-1.0-0 libnss3 libxshmfence1 libgbm1 libxfixes3

WORKDIR /app
COPY --from=builder /app/target/release/l4l /app/l4l
CMD ["/app/l4l"]