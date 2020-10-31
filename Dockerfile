FROM rust:latest
RUN /bin/bash -c 'apt-get update'
RUN /bin/bash -c 'apt-get -y upgrade'
WORKDIR /usr/src/market_watcher
COPY . .
RUN mkdir /var/log/market_watcher
RUN cargo install --path .
# CMD coinbase_client_01 2>&1 | tee -a /log/crypto_mon_$(date '+%Y%m%d%S').log
CMD market_watcher