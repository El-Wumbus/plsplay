INSTALL_LOCATION = /usr/bin
BIN = plsplay

default: build

build:
	cargo build

install:
	cargo build --release
	sudo install target/release/$(BIN) $(INSTALL_LOCATION)/$(BIN)

build_x86_linux:
	mkdir build
	cargo build --release
	tar -cvjf build/plsplay_x86_64-linux.tar.xz target/release/plsplay

clean:
	rm -rf ./target/* build