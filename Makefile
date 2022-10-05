INSTALL_LOCATION = /usr/bin
BIN = plsplay

default: build

build:
	cargo build

build-release:
	cargo build --release
	install target/release/$(BIN) $(INSTALL_LOCATION)/$(BIN)

clean:
	rm -rf ./target/*