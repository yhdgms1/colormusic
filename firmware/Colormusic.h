#include "Udp.h"

#ifndef COLORMUSIC_H
#define COLORMUSIC_H

void colormusicSetup();
void handleUdp(UDP& udp);
void writeColors(int* R, int* G, int* B);

#endif