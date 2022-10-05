# PLSPLAY

A CLI audio file playing utility.

## How to install

### Compile from source

1. Ensure you install [cargo](https://www.rust-lang.org/tools/install).
2. Have make installed too.
3. Clone the repo

   ```bash
    git clone https://github.com/el-wumbus/plsplay.git
    cd plsplay
   ```

4. Compile & Install

   ```bash
    # COOL PEOPLE
    sudo make install

    # Windows users
    install.ps1 # Not working yet, ok?
   ```

> Calm down, it's a joke.

## Usage

```bash
$ plsplay --help
plsplay 0.1.0

USAGE:
    plsplay [OPTIONS] <file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -v, --volume <volume>    The playback volume (from 0 to 100) [default: 100]

ARGS:
    <file>    The audio file to play
```

### Example

```bash
$ plsplay ~/cool_song.flac --volume 55
Playing '/home/user/cool_song.flac' at 55% volume
```
