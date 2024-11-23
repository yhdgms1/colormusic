#include <math.h>

double _lerp(double a, double b, double t) {
  return a + t * (b - a);
}

void oklch_lerp(const double *OKLCH1, const double *OKLCH2, double *OUT, double t) {
  OUT[0] = _lerp(OKLCH1[0], OKLCH2[0], t);
  OUT[1] = _lerp(OKLCH1[1], OKLCH2[1], t);

  double hue1 = OKLCH1[2];
  double hue2 = OKLCH2[2];
  double delta = hue2 - hue1;
  
  // Shortest hue path around the circle
  if (fabs(delta) > 180.0) {
    delta -= copysign(360.0, delta);
  }
  
  OUT[2] = hue1 + t * delta;
}
