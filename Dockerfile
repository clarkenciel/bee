FROM debian:bookworm-slim AS tini

RUN apt update && apt install -y curl

WORKDIR /tini

# Add Tini
ENV TINI_VERSION=v0.19.0
RUN curl -L -o ./tini \
  "https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini-amd64" \
  && chmod +x ./tini

FROM debian:bookworm-slim

RUN \
  groupadd --gid 10001 --system bee && \
  useradd --uid 10001 -d /home/bee -b /home -g bee --system --create-home bee

WORKDIR /home/bee

COPY --chown=bee: --chmod=0100 --from=tini /tini/tini ./tini
COPY --chown=bee: --chmod=0400 ./frontend/dist/index.html ./index.html
COPY --chown=bee: --chmod=0700 ./frontend/dist/ ./assets/
COPY --chown=bee: --chmod=0700 ./.sqlx/ ./.sqlx/
COPY --chown=bee: --chmod=0100 ./target/release/server ./server

USER bee

ENTRYPOINT ["./tini", "--", "./server"]
CMD []
