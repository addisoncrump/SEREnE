FROM ekidd/rust-musl-builder as build

RUN cargo init
ADD Cargo.lock Cargo.toml ./

RUN cargo build --release

RUN rm -rf src
COPY src /home/rust/src/src
RUN cargo build --release

FROM scratch

COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/serene /serene

ENTRYPOINT ["/serene"]
