#include <Wire.h>

unsigned long lastTime = 0;

// angle tracking vars
const int MPU6050_addr = 0x68;
int16_t GyroZ;
bool gyroReady = false;
void setGyroReady() { gyroReady = true; }
float angleZ = 0.00;
float gyroZBias = 0.00;
float q = 0.01;
float r = 1.00;
float p = 0.10;
float x = 0.00;
float k = 0.00;

// rotary encoder tracking vars
const int numberEncoders = 4;
const int rotaryInputPins[4] = { 4, 6, 8, 10 };
const int rotaryClockPins[4] = { 3, 5, 7, 9 };
int rotaryPositions[4] = { 0, 0, 0, 0 };
bool rotaryStates[4] = { false, false, false, false };

void setup() {
  // setup rotary pings
  for (int i = 0; i < numberEncoders; i++) {
    pinMode(rotaryInputPins[i], INPUT);
    pinMode(rotaryClockPins[i], INPUT);
  }
 
  // setup gyroscope
  Wire.setClock(400000);
  Wire.begin();

  // enable MPU 6050
  Wire.beginTransmission(MPU6050_addr);
  Wire.write(0x6B); Wire.write(0);
  Wire.endTransmission(true);

  // set 125hz sample rate (1000 / (1 + 7))
  Wire.beginTransmission(MPU6050_addr);
  Wire.write(0x19); Wire.write(0x07);
  Wire.endTransmission(true);

  // set DLPF to 3
  Wire.beginTransmission(MPU6050_addr);
  Wire.write(0x1A); Wire.write(0x03);   
  Wire.endTransmission(true);

  // enable mpu interupt
  Wire.beginTransmission(MPU6050_addr);
  Wire.write(0x38); Wire.write(0x01);   
  Wire.endTransmission(true);

  // configure mpu interupt
  Wire.beginTransmission(MPU6050_addr);
  Wire.write(0x37); Wire.write(0x00);   
  Wire.endTransmission(true);

  // attach mpu interupt
  attachInterrupt(digitalPinToInterrupt(2), setGyroReady, RISING);

  // start serial
  Serial.begin(9600);
  delay(1000);

  // Calibrate GyroZ bias (assumes MPU6050 is stationary)
  long sum = 0;
  const int samples = 500;
  for (int i = 0; i < samples; i++) {
    Wire.beginTransmission(MPU6050_addr);
    Wire.write(0x47); // GyroZ high byte register
    Wire.endTransmission(false);
    Wire.requestFrom(MPU6050_addr, 2, true);
    int16_t gz = Wire.read() << 8 | Wire.read();
    sum += gz;
    delay(2);
  }
  gyroZBias = sum / (float)samples;
  Serial.println(gyroZBias);
 
  lastTime = millis();
}

uint8_t buffer[14];

void loop() {
  unsigned long time = millis();
  unsigned long tick_length = time - lastTime;
  lastTime = time;

  if (gyroReady) {
    gyroReady = false;
    
    // read gyro info
    Wire.beginTransmission(MPU6050_addr);
    Wire.write(0x3B);
    Wire.endTransmission(false);
    Wire.requestFrom(MPU6050_addr, 14, true);
    Wire.readBytes(buffer, 14);
    GyroZ = buffer[12] << 8 | buffer[13];
  }

  // apply kalman filter
  p = p + q;
  k = p / (p + r);
  x = x + k * (GyroZ - gyroZBias - x);
  p = (1 - k) * p;

  // calculate Z axis
  float dt = tick_length / 1000.0;
  angleZ += x / 131.0 * dt;

  // read all encoders
  for (int i = 0; i < numberEncoders; i++)
    updateEncoder(i);

  // only read out when requested
  if (Serial.available()) {
    // clear read buffer
    while (Serial.available()) Serial.read();

    // print debug output
    byte* angle_bytes = (byte*) &angleZ;
    Serial.write(angle_bytes, 4);
    byte* ne_bytes = (byte*) &numberEncoders;
    Serial.write(ne_bytes, 4);
    for (int pos : rotaryPositions) {
      byte* pos_bytes = (byte*) &pos;
      Serial.write(pos_bytes, 2);
    }
  }
}

void updateEncoder(int idx) {
  bool clk = digitalRead(rotaryClockPins[idx]);
  bool dt = digitalRead(rotaryInputPins[idx]);

  if (rotaryStates[idx] != clk) {
    if (clk != dt) {
      rotaryPositions[idx] ++;
    } else {
      rotaryPositions[idx] --;
    }
  }

  rotaryStates[idx] = clk;
}
