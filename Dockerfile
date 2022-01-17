FROM rust:1.58 as builder

WORKDIR /webapi

COPY . .

RUN cargo build --release
RUN ls -a

FROM debian:buster-slim
ARG APP=/var/webapi

RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config build-essential

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

EXPOSE 8080
COPY --from=builder /webapi/target/release/povorot-web-api ${APP}

RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}
CMD [ "./povorot-web-api" ]