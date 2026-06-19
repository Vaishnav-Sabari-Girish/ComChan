#include <HardwareSerial.h>

// Initialize Hardware UART 2
HardwareSerial commSerial(2); 

// Standard ESP32 UART2 pins
#define RX_PIN 16
#define TX_PIN 17

void setup() {
  // Start the hardware serial for your PC console
  Serial.begin(115200);
  
  // Start UART2 at 9600 baud, 8 data bits, no parity, 1 stop bit
  commSerial.begin(9600, SERIAL_8N1, RX_PIN, TX_PIN);
  
  Serial.println("\n--- Bharat Pi UART Chat Started ---");
  Serial.println("Type here to send to ESP8266.");
}

void loop() {
  // If data comes FROM the ESP8266, print it to the PC
  if (commSerial.available()) {
    char c = commSerial.read();
    Serial.write(c);
  }

  // If you type data IN the PC console, send it to the ESP8266
  if (Serial.available()) {
    char c = Serial.read();
    commSerial.write(c);
  }
}
