#define ledPin 13

void setup() {
  pinMode(ledPin, OUTPUT);
  Serial.begin(9600); // Make sure baud rate matches
  Serial.println("Arduino Ready");
}

void loop() {
  delay(1000);
  Serial.println("Hello World");
}
