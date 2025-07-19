target/release/server:
	cargo build -p server --release

frontend/dist:
	cd frontend && trunk build --release

image: target/release/server frontend/dist
	docker build -f Dockerfile -t bee:latest .

.PHONY: clean

clean:
	cargo clean && cd frontend && trunk clean
