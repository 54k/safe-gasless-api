FROM --platform=linux/amd64 rust:1.64 as builder

RUN USER=root cargo new --bin smartwallet-api
WORKDIR ./smartwallet-api
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm -rf src

COPY ./src ./src
COPY ./abi ./abi

RUN rm ./target/release/deps/smartwallet_api*
RUN cargo build --release


FROM --platform=linux/amd64 debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8080

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /smartwallet-api/target/release/smartwallet-api ${APP}/smartwallet-api

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

COPY ./.env.prod ./.env

CMD ["./smartwallet-api"]