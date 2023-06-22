# syntax=docker/dockerfile:1.5
FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:${TARGETARCH}-musl as builder

ARG NAME=cqrs-es-example-write-api
ARG TARGETARCH

RUN if [ $TARGETARCH = "amd64" ]; then \
      echo "x86_64" > /arch; \
    elif [ $TARGETARCH = "arm64" ]; then \
      echo "aarch64" > /arch; \
    else \
      echo "Unsupported platform: $TARGETARCH"; \
      exit 1; \
    fi

COPY Cargo.toml .

ADD --chown=rust:rust infrastructure infrastructure
ADD --chown=rust:rust rmu rmu

ADD --chown=rust:rust command/write-api command/write-api
ADD --chown=rust:rust command/domain command/domain
ADD --chown=rust:rust command/processor command/processor
ADD --chown=rust:rust command/interface-adaptor command/interface-adaptor

ADD --chown=rust:rust query/read-api query/read-api
ADD --chown=rust:rust query/interface-adaptor query/interface-adaptor
ADD --chown=rust:rust query/processor query/processor

RUN CARGO_BUILD_INCREMENTAL=true cargo build -p ${NAME} --release --target $(cat /arch)-unknown-linux-musl \
    && cp target/$(cat /arch)-unknown-linux-musl/release/${NAME} target/release/app

FROM --platform=$TARGETPLATFORM alpine
RUN apk --no-cache add ca-certificates
COPY --from=builder /home/rust/src/target/release/app /app
CMD [ "/app" ]