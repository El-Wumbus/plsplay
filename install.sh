#!/usr/bin/env sh

cargo build --release || echo "Couldn't compile program!"
exit 1

clean() {
    rm -rf ./target
}

if [ "$(uname)" = "darwin" ]; then
    install_path="${PREFIX}${HOME}/.local/bin/" # Install to a user local directory because macos tries to hide it's important directories 
    chmod 755 target/release/plsplay
    mkdir -p "$install_path"
    cp -vf target/release/plsplay "$install_path" 

else
    install_path="${PREFIX}/usr/local/bin/"
    mkdir -p "$install_path"
    sudo install -Dm755 target/release/plsplay "$install_path/plsplay"
fi

clean

if [ "$(echo "$PATH" | grep -i "$install_path" )" = "" ]; then
    echo "'${install_path}' isn't in your PATH, add it to use plsplay."
fi