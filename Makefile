INSTALL_LOCATION = /usr/bin
BIN = plsplay

default: build

build:
	cargo build

install:
	cargo build --release
	sudo install target/release/$(BIN) $(INSTALL_LOCATION)/$(BIN)

clean:
	rm -rf ./target/*