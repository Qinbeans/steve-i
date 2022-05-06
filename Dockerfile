FROM rust:slim AS builder

RUN apt-get update && apt-get install build-essential default-libmysqlclient-dev autoconf automake libtool \
    m4 libopus-dev libssl-dev pkg-config -y

WORKDIR /steve

COPY ./src/ ./src/
COPY ./diesel.toml ./diesel.toml
COPY ./migrations ./migrations
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
# copy builds to target
COPY ./builds ./target

RUN cargo build --release

FROM rust:slim AS runtime
RUN apt-get update 
RUN apt-get install -y build-essential autoconf automake libtool m4 libmariadb-dev libpq-dev libsqlite3-dev \
    ffmpeg libopus-dev software-properties-common python3-pip default-libmysqlclient-dev python3-venv bash \
    libssl-dev

# non root user
RUN useradd -m -N steve

RUN pip install --upgrade pip && pip3 install youtube-dl
RUN cargo install diesel_cli --no-default-features --features mysql

USER steve
RUN python3 -m venv /home/steve/.venv && find /home/steve/.venv -name activate
RUN bash -c 'source /home/steve/.venv/bin/activate'
RUN python3 -V && youtube-dl --version && ffmpeg -version && diesel --version

COPY --from=builder /steve/target/release/steve /usr/local/bin/steve