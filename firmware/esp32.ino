#include <SPI.h>
#include <Arduino.h>
#include <WiFi.h>
#include <WiFiUdp.h>
#include "Colormusic.h"

#define D2_LED_PIN 2

// WiFi
const char* ssid = "network";
const char* password = "password";

// UDP Port
const uint16_t LOCAL_PORT = 8488;

// GPIO Pins
const int R = 16;
const int G = 17;
const int B = 18;

// PWM Config
const int pwmFreq = 5000;
const int pwmResolution = 8;

WiFiUDP udp;

void setup() {
  WiFi.begin(ssid, password);

  while(WiFi.status() != WL_CONNECTED){
    delay(100);
  }

  udp.begin(LOCAL_PORT);

  // Disable annoying LED
  pinMode(D2_LED_PIN, INPUT);

  // Setup LED
  ledcAttach(R, pwmFreq, pwmResolution);
  ledcAttach(G, pwmFreq, pwmResolution);
  ledcAttach(B, pwmFreq, pwmResolution);

  colormusicSetup();
}

void loop() {
  int r, g, b;

  handleUdp(udp);
  writeColors(&r, &g, &b);

  ledcWrite(R, r);
  ledcWrite(G, g);
  ledcWrite(B, b); 
}