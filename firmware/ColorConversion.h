#ifndef COLORCONVERSION_H
#define COLORCONVERSION_H

void oklch2rgb(const double *LCH, int *OUT);
void rgb2oklch(const int *RGB, double *OUT);

#endif