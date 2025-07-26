psql:
    docker compose run --rm -it --entrypoint 'sh -c' postgres -- 'psql -U bee -h postgres -p 5432'
