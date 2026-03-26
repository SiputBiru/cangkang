FROM rust:1.94.0-slim-trixie AS builder

WORKDIR /usr/src/cangkang

COPY . .

RUN cargo build --release && ./target/release/cangkang

FROM nginx:1.28.3-alpine

RUN rm /etc/nginx/conf.d/default.conf
COPY nginx.conf /etc/nginx/conf.d/default.conf

RUN rm -rf /usr/share/nginx/html/*

COPY --from=builder /usr/src/cangkang/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
