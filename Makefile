build:
	@BUILD_ICONS=1 cargo build

lint:
	@cargo clippy --all-targets --features icon --features markdown --features minify --features with-axum --tests --benches -- -D warnings

test:
	@cargo nextest run --features icon --features markdown --features minify --features with-axum

release:
	@cargo release tag --execute
	@git cliff -o CHANGELOG.md
	@git commit -a -n -m "Update CHANGELOG.md" || true
	@git push origin master
	@cargo release push --execute

update-submodule:
	@git submodule update --init --recursive --remote

.PHONY: build test release update-submodule
