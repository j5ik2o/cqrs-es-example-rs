FROM bash:devel-alpine3.19

RUN apk add --no-cache curl jq

COPY ./common.sh ./verify-group-chat.sh ./verify-after-restart.sh ./

ENTRYPOINT ["./verify-group-chat.sh"]
