FROM rust:1.47-slim

WORKDIR /var/www/leftgang
COPY . .

RUN rustc --version
RUN cargo install --path /var/www/leftgang

EXPOSE 3000

CMD ["leftgang"]