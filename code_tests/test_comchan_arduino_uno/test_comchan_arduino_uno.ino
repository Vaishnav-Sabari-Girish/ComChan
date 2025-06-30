#define ledPin 13

void setup() {
  pinMode(ledPin, OUTPUT);
  Serial.begin(9600); // Make sure baud rate matches
  Serial.println("Arduino Ready");
}

void loop() {
  delay(1000);
  Serial.println("Hello World");
  while (Serial.available()) {

    String data = Serial.readStringUntil('\n');
    data.trim(); // Remove whitespace/newlines
    Serial.print("Received: ");
    Serial.println(data);

    if (data == "ON" || data == "on"){
      digitalWrite(ledPin, 1);
    }
    else if(data == "OFF" || data == "off"){
      digitalWrite(ledPin, 0);
    }
    else {
      digitalWrite(ledPin, 0);
    }
  }
}
