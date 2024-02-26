FROM rust:1.76.0 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .
CMD ["quotes-tg-bot"]