psql:
    docker compose run --rm -it --entrypoint 'sh -c' postgres -- 'psql -U bee -h postgres -p 5432'
be $BEE_LOG_LEVEL="DEBUG":
    cargo watch -w server -x 'run -p server'
