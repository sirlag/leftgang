FROM rust:1.47-slim as cargo-build

WORKDIR /usr/src/leftgang

COPY . .

RUN rustc --version
RUN cargo build --release

FROM debian:buster-slim

WORKDIR /var/www/leftgang

COPY --from=cargo-build /usr/src/leftgang/target/release/leftgang /var/www/leftgang

EXPOSE 3000

RUN ls
CMD ["/var/www/leftgang/leftgang"]