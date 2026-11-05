# Build stage: bookworm matches the runtime's glibc
FROM rust:1-bookworm AS builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN useradd --system --no-create-home --shell /usr/sbin/nologin ferisss

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/ferisss ./ferisss
# Rocket.toml sets address = 0.0.0.0, so no ROCKET_ADDRESS is needed
COPY Rocket.toml ./

USER ferisss

EXPOSE 8000

CMD ["./ferisss"]
