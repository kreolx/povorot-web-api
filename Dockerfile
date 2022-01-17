FROM rust:1.58 as builder

WORKDIR /webapi

COPY . .

RUN cargo build --release

FROM debian:buster-slim
ARG APP=/var/webapi

ENV TZ=Etc/UTC \
    APP_USER=appuser

EXPOSE 3000
COPY --from=builder /webapi/target/release/webapi ${APP}

RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}
CMD [ "./webapi" ]