FROM rust:1-slim-trixie AS dev
WORKDIR /app
# o compose roda como o usuário do host; o cache do cargo precisa ser gravável por ele
RUN mkdir -p /usr/local/cargo/registry && chmod -R a+rwX /usr/local/cargo

FROM dev AS build
COPY . .
RUN cargo build --release --locked

FROM debian:trixie-slim AS runtime
COPY --from=build /app/target/release/sebo-do-lima /usr/local/bin/sebo-do-lima
USER nobody
CMD ["sebo-do-lima"]
