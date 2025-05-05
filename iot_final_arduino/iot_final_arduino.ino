#include <Wire.h>

// time trackers
unsigned long lastTime = 0;
unsigned long lastGyroUpdate = 0;
unsigned long tick_length = 0;
unsigned long longest_length = 0;

// angle tracking vars
const int MPU6050_addr = 0x68;
uint8_t wire_buffer[14];
int16_t GyroZ;
float angleZ = 0.00;
float gyroZBias = 0.00;
float q = 0.01;
float r = 1.00;
float p = 0.10;
float x = 0.00;
float k = 0.00;

// rotary encoder vars
const int NUM_ENCODERS = 4;
const int clockPins[4] = { 7, 8, 9, 10 };
const int dataPins[4] = { 3, 4, 5, 6 };
int encoderPositions[4] = { 0, 0, 0, 0 };
bool encoderStates[4] = { false, false, false, false };

// button vars
const int NUM_BUTTONS = 6;
const int buttonPins[6] = { A0, A1, A2, A3, 11, 12 };

// interrupt handling
bool gyroReady = false;
void setGyroReady() { gyroReady = true; }

void setup() {
  // setup clock and data pins for rotary encoders
  for (int i = 0; i < NUM_ENCODERS; i++) {
    pinMode(clockPins[i], INPUT);
    pinMode(dataPins[i], INPUT);
  }

  for (int i = 0; i < NUM_BUTTONS; i++) {
    pinMode(buttonPins[i], INPUT_PULLUP);
  }

  // setup PCINT interrupts
  PCICR |= (1 << PCIE2);
  PCMSK2 |= (1 << PCINT23);
  PCMSK2 |= (1 << PCINT19);
  PCMSK2 |= (1 << PCINT0);
  PCMSK2 |= (1 << PCINT20);
  PCMSK2 |= (1 << PCINT1);
  PCMSK2 |= (1 << PCINT21);
  PCMSK2 |= (1 << PCINT2);
  PCMSK2 |= (1 << PCINT22);

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

  // complete startup
  Serial.begin(115200);
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

  lastTime = micros();
  lastGyroUpdate = micros();
}

void loop() {
  unsigned long time = micros();
  tick_length = time - lastTime;
  lastTime = time;
  if (tick_length > longest_length) longest_length = tick_length;

  if (gyroReady) {
    gyroReady = false;
    readGyroscope();
  }

  if (Serial.available()) {
    while(Serial.available()) {
      Serial.read();
    }
//     writeStdOutput();
    writeBinOutput();
  }
}

ISR(PCINT2_vect) {
  for (int i = 0; i < NUM_ENCODERS; i++) {
    bool clk = digitalRead(clockPins[i]);
    bool dt = digitalRead(dataPins[i]);

    if (clk != encoderStates[i]) {
      if (dt != clk) {
        encoderPositions[i]++;
      } else {
        encoderPositions[i]--;
      }
      encoderStates[i] = clk;
    }
  }
}

/**
 * Reads the raw gyroscope input from the Z axis
 */
int16_t readRawGyroscope() {
  Wire.beginTransmission(MPU6050_addr);
  Wire.write(0x3B);
  Wire.endTransmission(false);
  Wire.requestFrom(MPU6050_addr, 14, true);
  Wire.readBytes(wire_buffer, 14);
  return wire_buffer[12] << 8 | wire_buffer[13];
}

/**
 * Read the gyroscope input and update the kalman filter and final rotation values.
 */
void readGyroscope() {
  GyroZ = readRawGyroscope();

  // apply kalman filter
  p = p + q;
  k = p / (p + r);
  x = x + k * (GyroZ - gyroZBias - x);
  p = (1 - k) * p;

  unsigned long time = micros();
  unsigned long last_gyro_length = time - lastGyroUpdate;
  lastGyroUpdate = time;

  // calculate Z axis
  float dt = last_gyro_length / 1000000.0;
  angleZ += x / 131.0 * dt;
}

/**
 * Writes the standard (debug) output to the serial port.
 */
void writeStdOutput() {
  Serial.print(tick_length);
  Serial.print(" | ");
  Serial.print(angleZ);

  for (int i = 0; i < NUM_ENCODERS; i++) {
    Serial.print(" | ");
    Serial.print(encoderPositions[i]);
  }
  
  Serial.println("");
}

/**
 * Writes the binary output to the serial port.
 */
void writeBinOutput() {
  // write header
  Serial.write(255);
  Serial.write((byte*) &tick_length, 4);
  Serial.write((byte*) &longest_length, 4);
  Serial.write((byte*) &angleZ, 4);

  // write encoders
  Serial.write((byte*) &NUM_ENCODERS, 4);
  for (int i = 0; i < NUM_ENCODERS; i++) {
    int pos = encoderPositions[i];
    Serial.write((byte*) &pos, 2);
  }

  Serial.write((byte*) &NUM_BUTTONS, 4); 
  for (int i = 0; i < NUM_BUTTONS; i++) {
    byte input = 0x01;
    if (digitalRead(buttonPins[i])) input = 0x00;
    Serial.write(input);
  }
}
