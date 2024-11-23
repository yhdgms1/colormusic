#include <SPI.h>
#include <Ethernet.h>
#include <EthernetUdp.h>
#include "Colormusic.h"

byte mac[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xED };
IPAddress ip(192, 168, 1, 167);

const unsigned int LOCAL_PORT = 8488;

const int R = 5;
const int G = 6;
const int B = 9;

EthernetUDP udp;

void setup() {
  pinMode(R, OUTPUT);
  pinMode(G, OUTPUT);
  pinMode(B, OUTPUT);

  colormusicSetup(R, G, B);

  Serial.begin(115200);
  Ethernet.begin(mac, ip);
  udp.begin(LOCAL_PORT);
}

void loop() {
  int r, g, b;

  handleUdp(udp);
  writeColors(&r, &g, &b);

  analogWrite(R, RGB[0]);
  analogWrite(G, RGB[1]);
  analogWrite(B, RGB[2]);
}