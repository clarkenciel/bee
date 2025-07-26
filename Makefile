.sqlx:
	cargo sqlx prepare --workspace

target/release/server: server/src/*.rs Cargo.* .sqlx
	cargo build -p server --release

frontend/dist: frontend/src/*.rs frontend/input.css frontend/Trunk.toml frontend/assets/* Cargo.*
	cd frontend && trunk build --release

image: target/release/server frontend/dist
	docker build -f Dockerfile -t bee:latest .

.PHONY: clean fe-clean

clean: fe-clean
	cargo clean

fe-clean:
	cd frontend && trunk clean
