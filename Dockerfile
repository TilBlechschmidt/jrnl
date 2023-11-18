FROM alpine

RUN apk --no-cache add ca-certificates

COPY target/x86_64-unknown-linux-musl/release/jrnl /jrnl
COPY frontend/build /frontend/build

CMD /jrnl