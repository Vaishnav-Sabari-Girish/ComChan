![Banner](./docs/src/images/banner.png)

---

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
<!--**Table of Contents**  *generated with [DocToc](https://github.com/thlorenz/doctoc)*-->

- [ComChan](#comchan)
  - [Installation](#installation)
    - [From crates.io](#from-cratesio)
    - [From source](#from-source)
  - [Common Commands](#common-commands)
    - [Basic Serial Monitor](#basic-serial-monitor)
    - [Verbose Mode](#verbose-mode)
    - [Log Mode](#log-mode)
  - [Features](#features)
    - [Legends](#legends)
- [Examples](#examples)
  - ["Hello World" Program](#hello-world-program)
  - [User Input](#user-input)
- [Feedback Form](#feedback-form)
- [Stargazers over time](#stargazers-over-time)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# ComChan

ComChan is a Blazingly Fast Serial monitor for Embedded Systems and Serial Communication. 

**Latest Version**: 0.1.4

## Installation

### From crates.io


> [!NOTE]
> `cargo install` NOW AVAILABLE

```bash
cargo install comchan

#Install the binary directly
cargo binstall comchan
```

After installing, check if it has been installed with

```bash
comchan --version
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

## Common Commands 

### Basic Serial Monitor

```bash
comchan -p <port> -r <baud_rate>

# OR

comchan --port <port> --baud <baud_rate>

```

### Verbose Mode

```bash
comchan -p <port> -r <baud_rate> -v

# OR

comchan --port <port> --baud <baud_rate> --verbose
```

### Log Mode

```bash
comchan -p <port> -r <baud_rate> -l <log_file_name>

# OR 

comchan --port <port> --baud <baud_rate> --log <log_file_name>
```

For an example log file , get it [here](./test.log)

## Features

- [x] Read incoming Serial data from Serial ports
- [x] Write to Serial port i.e Send data to Serial device.
- [x] Basic logging.
- [ ] Write serial data to a file for later use (can be .txt , .csv and more)
- [ ] Terminal based Serial Plotter (to be implemented with the `--plot` command)

### Legends

- [x] Implemented Features
- [ ] Yet to me implemented

# Examples

## "Hello World" Program

![GIF](./docs/src/videos/basic_serial_mon.gif)

## User Input

![User IP](./docs/src/videos/basic_user_input.gif)


# Feedback Form

The Feedback form was created using Bashforms (Forms in the terminal itself). 

To give you feedback, please type this on your terminal 

```bash
ssh -t bashform.me f comchan
```


                        
# Stargazers over time

[![Stargazers over time](https://starchart.cc/Vaishnav-Sabari-Girish/ComChan.svg?variant=dark)](https://starchart.cc/Vaishnav-Sabari-Girish/ComChan)

                    
