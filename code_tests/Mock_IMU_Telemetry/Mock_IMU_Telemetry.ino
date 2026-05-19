void setup() {
  // Set the baud rate to match ComChan's high-speed default
  Serial.begin(115200);

  // Wait for the serial port to connect (helpful for boards with native USB like Leonardo/ESP32)
  while (!Serial) {
    delay(10);
  }
}

void loop() {
  // Get the current time in seconds to drive the smooth math
  float time_sec = millis() / 1000.0;

  // Use sine and cosine to simulate a smoothly tumbling object
  float pitch = sin(time_sec * 0.5) * 45.0; // Pitches back and forth +/- 45 degrees
  float roll  = cos(time_sec * 0.8) * 30.0; // Rolls side to side +/- 30 degrees

  // Yaw spins continuously, so we let time drive it and wrap it at 360
  float yaw   = time_sec * 45.0; // Spins at 45 degrees per second
  yaw = fmod(yaw, 360.0);
  if (yaw > 180.0) yaw -= 360.0; // Normalize between -180 and +180

  // Format the output exactly how ComChan expects it to parse the hashmap: "Name: Value, Name: Value"
  Serial.print("Pitch: ");
  Serial.print(pitch);
  Serial.print(", Yaw: ");
  Serial.print(yaw);
  Serial.print(", Roll: ");
  Serial.println(roll);

  // Delay 50ms for a smooth 20 frames per second update rate
  delay(50);
}
