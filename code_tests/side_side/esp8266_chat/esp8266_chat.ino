#include <SoftwareSerial.h>

// RX = D1 (GPIO 5), TX = D2 (GPIO 4)
SoftwareSerial commSerial(5, 4); 

void setup() {
  // Start the hardware serial for your PC console (picocom)
  Serial.begin(115200);
  
  // Start the software serial to talk to the Bharat Pi
  commSerial.begin(9600);
  
  Serial.println("\n--- ESP8266 UART Chat Started ---");
  Serial.println("Type here to send to Bharat Pi.");
}

void loop() {
  // If data comes FROM the Bharat Pi, print it to the PC
  if (commSerial.available()) {
    char c = commSerial.read();
    Serial.write(c);
  }

  // If you type data IN the PC console, send it to the Bharat Pi
  if (Serial.available()) {
    char c = Serial.read();
    commSerial.write(c);
  }
}
