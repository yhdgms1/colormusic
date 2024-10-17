#include <SPI.h>
#include <Ethernet.h>
#include <EthernetUdp.h>

byte mac[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xED };
IPAddress ip(192, 168, 1, 167);
unsigned int LOCAL_PORT = 8488;

const int R = 5;
const int G = 6;
const int B = 9;

int r = 0, g = 0, b = 0;

EthernetUDP udp;
char packetBuffer[UDP_TX_PACKET_MAX_SIZE];

void setup() {
  pinMode(R, OUTPUT);
  pinMode(G, OUTPUT);
  pinMode(B, OUTPUT);
  
  Ethernet.begin(mac, ip);
  udp.begin(LOCAL_PORT);

  // Serial.begin(115200);
  // Serial.println("UDP Client ready");
}

void loop() {
  int packetSize = udp.parsePacket();

  if (packetSize) {
    udp.read(packetBuffer, UDP_TX_PACKET_MAX_SIZE);
    
    // Serial.println(packetBuffer);

    String response = String(packetBuffer);

    sscanf(response.c_str(), "%d %d %d", &r, &g, &b);
    
    analogWrite(R, r);
    analogWrite(G, g);
    analogWrite(B, b);

    memset(packetBuffer, 0, UDP_TX_PACKET_MAX_SIZE);
  }
}
