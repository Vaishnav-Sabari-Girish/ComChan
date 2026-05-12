void setup() {
  Serial.begin(9600);
}

void loop() {
  // Print all sensors on a single line, separated by commas.
  // Only the VERY LAST print should be a println!
  
  Serial.print("Accelerometer:");
  Serial.print(random(100)); 
                             
  Serial.print(", Gyroscope:");
  Serial.print(random(100));
  
  Serial.print(", Magnetometer:");
  Serial.println(random(100));
  
  
  delay(1000);
}
