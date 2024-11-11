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
  colormusicSetup(R, G, B);

  Serial.begin(115200);
  Ethernet.begin(mac, ip);
  udp.begin(LOCAL_PORT);
}

void loop() {
  handleUdp(udp);
  writeColors(R, G, B);
}