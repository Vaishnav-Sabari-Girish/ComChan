
void setup() {
  Serial.begin(9600);
}

void loop() {
  Serial.print("Magnetometer : ");
  Serial.println(random(100));
  Serial.print("Gyroscope : ");
  Serial.println(random(100));
  Serial.print("Accelerometer : ");
  Serial.println(random(100));
  delay(1000);
}
