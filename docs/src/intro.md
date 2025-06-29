# 📘 Introduction to Comchan

**Comchan** (short for *"Communication Channel"*) is a **blazingly fast, minimal, and beginner-friendly serial monitor** made with 💖 in Rust.

It’s built for makers, tinkerers, students, embedded developers — **anyone** who works with serial-connected devices like **Arduino**, **ESP32**, **Teensy**, or **Raspberry Pi**, and wants a clean, modern, and reliable way to talk to them from the terminal.

> 🚀 Whether you're debugging a sensor or sending messages to a microcontroller — *Comchan* is your calm, capable companion.

---

## ✨ Why Comchan?

Most serial tools out there are either:

- ❌ Too basic (like `screen`, which doesn’t handle inputs well)
- ❌ Too bloated (like GUIs you don’t need)
- ❌ Confusing for newcomers

Comchan is:

- 🧼 **Minimal** – Does one thing and does it well
- ⚡ **Fast** – Built in Rust with snappy performance
- 🎨 **Pretty** – Uses emoji + colored output for clarity
- 🧠 **Smart** – Handles line buffering, timeouts, and clean exits
- ✍️ **Simple to Use** – Just tell it your port and baud rate and you’re set!

---

## 🔧 What Can It Do?

- 📥 **Read data from your Arduino or ESP32**
- 📤 **Send messages directly to your device**
- 🧃 **See real-time communication** as it happens
- 🎨 **Colorful logs** that are clean and easy to follow
- 🙋‍♂️ **Beginner-safe** – Doesn’t crash on common mistakes

---

## 🛠 Sample Use

```bash
# For Linux Users 
comchan -p /dev/ttyUSB0 -r 9600

# OR 

comchan --port /dev/ttyUSB0 --baud 9600

# For Windows Users 
comchan -p COM3 -r 9600

# OR 

comchan --port COM3 --baud 9600
```

## 💡 The Goal

Comchan was built to make embedded development more joyful, less frustrating, and a little bit cute 🐣.
Whether you're writing your first `Serial.println("Hello")`, or debugging complex protocols, Comchan will stay out of your way and do exactly what you ask.

_✨ Made with Rust and a little anime soul._
