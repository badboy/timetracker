.PHONY: build

BASE=/home/jer/webplatform

build:
	multirust override nightly-2015-03-12
	RUST_BACKTRACE=1 $(BASE)/cargo-build/target/debug/cargo-build --sysroot $(BASE)/rust-rt-minimal/sysroot/ --target i386-unknown-emscripten --emit em-js --release --verbose --emcc ./emcc
	cp target/i386-unknown-emscripten/release/webplatform.js static/js

watch:
	nodemon --watch src --exec "make || true" -e rs

serve:
	@echo 'http://0.0.0.0:8080/webplatform.html'
	@echo ''
	cd target/i386-unknown-emscripten/release/; hs
