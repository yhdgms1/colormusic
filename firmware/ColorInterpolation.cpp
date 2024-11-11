double lerp(double a, double b, double t) {
  return a + t * (b - a);
}

void oklch_lerp(const double *OKLCH1, const double *OKLCH2, double *OUT, double t) {
  OUT[0] = lerp(OKLCH1[0], OKLCH2[0], t);
  OUT[1] = lerp(OKLCH1[1], OKLCH2[1], t);
  OUT[2] = lerp(OKLCH1[2], OKLCH2[2], t);
}
