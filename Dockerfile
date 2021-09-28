FROM rustlang/rust:nightly-alpine as builder
WORKDIR /usr/src/imitari
RUN apk update \
    && apk add --no-cache \
    musl-dev 
COPY . .
RUN cargo install --all-features --path .

FROM alpine:edge
RUN apk update \
    && apk add --no-cache \
    ca-certificates \
    musl-dev \
    tini
RUN update-ca-certificates
WORKDIR /usr/local/bin
COPY --from=builder /usr/local/cargo/bin/imitari .
COPY --from=builder /usr/src/imitari/public ./public
COPY --from=builder /usr/src/imitari/templates ./templates
CMD ["imitari"]
