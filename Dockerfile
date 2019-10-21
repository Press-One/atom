FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build -j4 --release
RUN cargo install -j4 diesel_cli --no-default-features --features postgres

ADD https://github.com/vishnubob/wait-for-it/raw/master/wait-for-it.sh /wait-for-it.sh
# Add Tini
ENV TINI_VERSION v0.18.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini-static /tini
RUN chmod +x /tini /wait-for-it.sh

FROM rust:latest
COPY --from=builder \
    /usr/src/app/target/release/atom /app/atom
COPY --from=builder /usr/src/app/migrations /app/migrations
COPY --from=builder /tini /tini
COPY --from=builder /wait-for-it.sh /wait-for-it.sh
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/cargo/bin/diesel
ENTRYPOINT ["/tini", "--"]

WORKDIR /app
CMD ["/app/atom", "syncserver"]
