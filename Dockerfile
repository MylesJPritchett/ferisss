FROM rust:1.67 as builder

COPY . /usr/app
WORKDIR /usr/app

RUN cargo install --path .

FROM debian:buster-slim as runner
COPY --from=builder /usr/local/cargo/bin/ferisss /usr/local/bin/ferisss
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["ferisss"]
