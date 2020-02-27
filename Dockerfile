FROM registry.gitlab.com/rust_musl_docker/image:stable-latest as builder
WORKDIR /workdir
COPY . .

RUN cargo build -j4 --release --target=x86_64-unknown-linux-musl
RUN cargo install -j4 diesel_cli --target=x86_64-unknown-linux-musl --no-default-features --features postgres

ADD https://github.com/eficode/wait-for/raw/8d9b4446df0b71275ad1a1c68db0cc2bb6978228/wait-for /wait-for
ADD https://github.com/krallin/tini/releases/download/v0.18.0/tini-static /tini
RUN chmod +x /tini /wait-for

FROM alpine
COPY --from=builder /workdir/target/x86_64-unknown-linux-musl/release/atom /app/atom
COPY --from=builder /workdir/migrations /app/migrations
COPY --from=builder /tini /tini
COPY --from=builder /wait-for /wait-for
COPY --from=builder /root/.cargo/bin/diesel /bin/diesel
ENTRYPOINT ["/tini", "--"]

WORKDIR /app
CMD ["/app/atom", "syncserver"]
