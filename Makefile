.PHONY: all build install clean lint check-fmt check-clippy

all: lint build 

build:
	cargo build --release

install:
	cargo install --path .
clean:
	cargo clean

lint: \
	check-fmt \
	check-clippy

check-fmt:
	cargo fmt --all --check

check-clippy:
	cargo clippy --no-deps --tests -- -D clippy::all
