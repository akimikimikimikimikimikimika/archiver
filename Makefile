build-debug:
	@`flags cargo debug`

build-macos: target.nosync/x86_64-apple-darwin/release/archiver target.nosync/aarch64-apple-darwin/release/archiver
	@lipo -create -output bin-macos $^

target.nosync/x86_64-apple-darwin/release/archiver: src/*.rs
	@`flags cargo macos x86_64`

target.nosync/aarch64-apple-darwin/release/archiver: src/*.rs
	@`flags cargo macos arm64`