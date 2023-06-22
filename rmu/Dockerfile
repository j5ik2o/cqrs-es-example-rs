# FROM rust:buster as build
FROM anyakichi/lambda-rust-builder:al2 AS build

ARG NAME=cqrs-es-example-rmu

COPY . /app
WORKDIR /app

RUN cargo build -p ${NAME} --release

FROM anyakichi/lambda-rust-builder:al2
COPY --from=build /app/target/release/cqrs-es-example-rmu ${LAMBDA_RUNTIME_DIR}/bootstrap
CMD [ "lambda-handler" ]
