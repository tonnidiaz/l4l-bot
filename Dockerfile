# Build stage
FROM rust:bookworm AS builder
 
WORKDIR /app
COPY . .
RUN cargo build --release
 
# Final run stage
FROM debian:bookworm-slim AS runner

RUN apt-get update -y && apt-get install -y libssl3 ca-certificates
RUN apt-get install -y \
libnss3 \
libdbus-1-3 \
libatk1.0-0 \
libasound2 \
libxrandr2 \
libxkbcommon-x11-0 \
libxfixes3 \
libxcomposite1 \
libxdamage1 \
libgbm1 \
libcups2 \
libcairo2 \
libpango-1.0-0 \
libatk-bridge2.0-0 \
libx11-6 \
libx11-xcb1 \
libxext6 \
libxi6 \
libxrender1 \
libxss1 \
libxtst6

WORKDIR /app
COPY --from=builder /app/target/release/l4l /app/l4l
CMD ["/app/l4l"]