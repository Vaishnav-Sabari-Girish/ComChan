# How to use ComChan

Currently in the latest version, ComChan has the following features

1. [Basic Serial Monitor](#basic-serial-monitor)
2. [Verbose Mode of Basic Serial Monitor](#verbose-mode)
3. [Logging data into a file](#logging)

## Basic Serial Monitor

ComChan can be used as a basic serial monitor for EMbedded Applications. 

In the Basic Serial Monitor mode, it has Read/Write capabilities. 

Here is a GIF showing how it works in basic mode : 

### Video 1 (Hello World printing)

We will run a basic Arduino application that continuously prints **Hello World**

**Apparatus Used**: 
1. Arduino Uno
2. ComChan

**Configuration**
1. Port: /dev/ttyACM0 (for Linux) **OR** COM3 (for windows)  (can be any number)
2. Baud Rate: 9600

**Command Used** :

```bash
comchan -p <port> -r <baud_rate>
```

<br>

![Basic mode Hello World](./videos/basic_serial_mon.gif)

<br>

### Video 2 (User Input)

We will now run another Arduino Application that takes User Input (1 or 0) to turn on and off the LED. 

**Working**: 

When the user types `on` or `ON`, the LED on the Arduino Turn ON and when the user types `off` or `OFF`, the LED turn OFF. 

Here are the GIF's :

<br>

<img src="./videos/basic_user_input.gif" width = "900">

<br>

<img src = "./videos/arduino.gif" width = "600">

## Verbose Mode

**Command Used** :

```bash
comchan -p <port> -r <baud_rate> -v 

# OR 

comchan --port <port> --baud <baud_rate> --verbose
```


ComChan also has a verbose mode where the timestamps are available.

The time stamps the time in milliseconds since January 1 1970 (Unix Epoch). 

Here is the GIF

<img src = "./videos/verbose_serial_mon.gif" width = "900">

<br> 

## Logging

**Command Used** : 

```bash
comchan -p <port> -r <baud_rate> -l <file_name>.log

# OR 

comchan --port <port> --baud <baud_rate> --log <file_name>.log
```

Log Files can be used to access the Serial Monitor data on a later date to debug Embedded Applications.

Here is the GIF of a Normal Serial Monitor

<img src = "./videos/logging_mon.gif" width = "900">

Here is the GIF of a log file

<img src = "./videos/logging_file.gif" width = "900">

You can access the sample log file [here](https://github.com/Vaishnav-Sabari-Girish/ComChan/blob/main/test.log).
