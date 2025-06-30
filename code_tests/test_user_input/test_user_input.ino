
#define ledPin 13

void setup() {
  pinMode(ledPin, OUTPUT);
  Serial.begin(9600);
}

void loop() {
  while (Serial.available()) {

    String data = Serial.readStringUntil('\n');
    data.trim(); // Remove whitespace/newlines
    
    // For Debugging
    //Serial.print("Received: ");
    //Serial.println(data);

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
