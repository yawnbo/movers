FROM rust as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin movers

FROM gcr.io/distroless/cc
#WORKDIR /app/config
#WORKDIR /app/cache
#WORKDIR /app
COPY --from=builder /app/target/release/movers .
CMD ["./movers"]
