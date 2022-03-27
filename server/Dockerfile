FROM rust:alpine as build

WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine
COPY --from=build /app/target/release/server ./server
CMD [./server]
