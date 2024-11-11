#include <SPI.h>
#include "Udp.h"
#include "ColorConversion.h"
#include "ColorInterpolation.h"

// Duration of color interpolation
int duration = 0;
// Start time (ms)
unsigned long startTime = 0;
// Incoming packet buffer
uint8_t buffer[5];

int RGB[3] = {0};

double OKLCH_PREV[3], OKLCH_CURR[3], OKLCH_OUT[3];

void colormusicSetup(int R, int G, int B) {
  pinMode(R, OUTPUT);
  pinMode(G, OUTPUT);
  pinMode(B, OUTPUT);

  rgb2oklch(RGB, OKLCH_PREV);
  rgb2oklch(RGB, OKLCH_CURR);
}

void handleUdp(UDP& udp) {
  int packetSize = udp.parsePacket();

  if (packetSize != 5) {
    return;
  }

  udp.read(buffer, 5);

  int r = buffer[0];
  int g = buffer[1];
  int b = buffer[2];
  duration = buffer[3] | (buffer[4] << 8);

  OKLCH_PREV[0] = OKLCH_CURR[0];
  OKLCH_PREV[1] = OKLCH_CURR[1];
  OKLCH_PREV[2] = OKLCH_CURR[2];

  RGB[0] = r;
  RGB[1] = g;
  RGB[2] = b;

  rgb2oklch(RGB, OKLCH_CURR);
  startTime = millis();

  memset(buffer, 0, 5);
}

void writeColors(int R, int G, int B) {
  unsigned long elapsedTime = millis() - startTime;

  // Progress in [0, 1] range
  double progress = fmin(1.0, (double)elapsedTime / duration);

  oklch_lerp(OKLCH_PREV, OKLCH_CURR, OKLCH_OUT, progress);
  oklch2rgb(OKLCH_OUT, RGB);
  
  analogWrite(R, RGB[0]);
  analogWrite(G, RGB[1]);
  analogWrite(B, RGB[2]);
}