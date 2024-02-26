FROM bash:devel-alpine3.19

RUN apk add --no-cache curl jq

ADD ./verify-group-chat.sh ./

ENTRYPOINT ["./verify-group-chat.sh"]
