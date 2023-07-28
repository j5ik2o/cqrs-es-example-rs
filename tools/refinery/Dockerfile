FROM ghcr.io/rust-db/refinery:main

RUN mkdir -p /app
WORKDIR /app
COPY ./migrations /app/migrations

ENTRYPOINT ["/usr/local/bin/refinery"]