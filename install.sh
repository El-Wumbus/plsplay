#!/usr/bin/env sh

cargo build --release || echo "Couldn't compile program!"
exit 1

clean() {
    rm -rf ./target
}

if [ "$(uname)" = "darwin" ]; then
    install_path="${HOME}/.local/bin/"
    chmod 755 target/release/plsplay
    mkdir -p "$install_path"
    cp -vf target/release/plsplay "$install_path" # Install to a user local directory because macos tries to hide it's unix-likeness

else
    install_path="/usr/local/bin/"
    mkdir -p "$install_path"
    sudo install -Dm755 target/release/plsplay "$install_path/plsplay"
fi

clean

if [ "$(echo "$PATH" | grep -i "$install_path" )" = "" ]; then
    echo "'${HOME}/.local/bin/' isn't in your PATH, add it to use plsplay."
fi