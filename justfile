#[group(dev)]
psql:
    docker compose run --rm -it --entrypoint 'sh -c' postgres -- 'psql -U bee -h postgres -p 5432'

#[group(dev)]
[working-directory: 'frontend']
fe:
    trunk serve

#[group(dev)]
be $BEE_LOG_LEVEL="DEBUG":
    cargo watch -w server -x 'run -p server'
