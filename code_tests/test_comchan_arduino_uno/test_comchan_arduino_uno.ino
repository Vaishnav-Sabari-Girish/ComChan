void setup() {
    Serial.begin(9600); // Make sure baud rate matches
    Serial.println("Arduino Ready");
}

void loop() {
    while (Serial.available()) {
        String data = Serial.readStringUntil('\n');
        data.trim(); // Remove whitespace/newlines
        Serial.print("Received: ");
        Serial.println(data);
    }
}
