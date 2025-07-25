# How to Install ComChan

There are multiple ways to install ComChan. 

## Install using `cargo install`

The easiest way is via `cargo`

```bash
# cargo install
cargo install comchan

#cargo binstall
cargo binstall comchan
```

Or you can download the binary from the latest [release](https://github.com/Vaishnav-Sabari-Girish/ComChan/releases/tag/v0.1.3)
To verify that `comchan` has been installed type the below command :

```bash
comchan --version

# OR

comchan --help
```

## Install from the AUR 

Thanks to [Orhun](https://github.com/orhun), ComChan now has an Arch package. You can install it using any package manager for arch like `yay` or `paru`

```bash
#Using yay 

yay -S comchan 

# Using paru

paru -S comchan
```

## Install using Homebrew 

ComChan can now be installed via Homebrew

```bash 
brew install Vaishnav-Sabari-Girish/taps/comchan
```


## Install from source

Clone the repository and `cd` in `ComChan`

```bash
cargo build --release
cargo run
```

