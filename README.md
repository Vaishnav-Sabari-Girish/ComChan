# ComChan

ComChan is a Blazingly Fast Serial monitor for Embedded Systems and Serial Communication. 

**Latest Version**: 0.1.0

## Installation

### From crates.io


> [!NOTE]
> `cargo install` NOW AVAILABLE

```bash
cargo install comchan

#Install the binary directly
cargo binstall comchan
```


### From source

```bash
# Clone from GitHub
git clone git@github.com:Vaishnav-Sabari-Girish/ComChan.git

# Clone from Codeberg
git clone ssh://git@codeberg.org/Vaishnav-Sabari-Girish/ComChan.git
```

```bash
cd ComChan

cargo build --release

cargo run
```

## Features

- [x] Read incoming Serial data from Serial ports
- [x] Write to Serial port i.e Send data to Serial device.
- [ ] Write serial data to a file for later use (can be .txt , .csv and more)
- [ ] Terminal based Serial Plotter (to be implemented with the `--plot` command)

### Legends

- [x] Implemented Features
- [ ] Yet to me implemented
