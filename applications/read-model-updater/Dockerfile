#ref: https://dev.classmethod.jp/articles/rust-app-container-on-lambda-function/
FROM public.ecr.aws/lambda/provided:al2 as build

ARG NAME=read-model-updater

RUN yum update -y && yum install -y gcc

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH $PATH:/root/.cargo/bin
RUN rustup install stable

RUN rustup target add x86_64-unknown-linux-gnu

COPY . /app
WORKDIR /app

ENV CARGO_BUILD_TARGET_DIR /tmp/target

RUN cargo build -p ${NAME} --release --target x86_64-unknown-linux-gnu

FROM public.ecr.aws/lambda/provided:al2
RUN mkdir -p /var/runtime && mkdir -p /tmp
COPY --from=build /tmp/target/x86_64-unknown-linux-gnu/release/read-model-updater /var/runtime/bootstrap
CMD [ "lambda-handler" ]