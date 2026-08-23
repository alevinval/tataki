.PHONY: lint
lint:
	cargo +nightly fmt
	cargo clippy --tests --fix --allow-dirty -- -Dclippy::all

.PHONY: test
test:
	cargo test
