#include "Udp.h"

#ifndef COLORMUSIC_H
#define COLORMUSIC_H

void colormusicSetup(int PIN_R, int PIN_G, int PIN_B);
void handleUdp(UDP& udp);
void writeColors(int PIN_R, int PIN_G, int PIN_B);

#endif