#include <SPI.h>
#include <math.h>

#define _PI 3.1415926535897932

void multiplyMatrices(const double *A, const double *B, double *OUT) {
  OUT[0] = A[0] * B[0] + A[1] * B[1] + A[2] * B[2];
  OUT[1] = A[3] * B[0] + A[4] * B[1] + A[5] * B[2];
  OUT[2] = A[6] * B[0] + A[7] * B[1] + A[8] * B[2];
}

void oklhch2oklab(const double *LCH, double *OUT) {
  double l = LCH[0];
  double c = LCH[1];
  double h = LCH[2];

  OUT[0] = l;
  OUT[1] = c * cos(h * _PI / 180);
  OUT[2] = c * sin(h * _PI / 180);
}

void oklab2oklch(const double *OKLab, double *OUT) {
  double l = OKLab[0];
  double a = OKLab[1];
  double b = OKLab[2];

  OUT[0] = l;
  OUT[1] = sqrt(a * a + b * b);

  if (fabs(a) < 0.0002 && fabs(b) < 0.0002) {
    OUT[2] = 0;
  } else {
    OUT[2] = fmod((atan2(b, a) * 180 / _PI + 360), 360);
  }
}

void rgb2srgbLinear(const double *RGBLinear, double *OUT) {
  for (unsigned short i = 0; i < 3; i++) {
    double c = RGBLinear[i];

    if (fabs(c) <= 0.04045) {
      OUT[i] = c / 12.92;
    } else {
      OUT[i] = (c < 0 ? -1 : 1) * pow((fabs(c) + 0.055) / 1.055, 2.4);
    }
  }
}

void srgbLinear2rgb(const double *RGB, double *OUT) {
  for (unsigned short i = 0; i < 3; i++) {
    double c = RGB[i];

    if (fabs(c) > 0.0031308) {
      OUT[i] = (c < 0 ? -1 : 1) * (1.055 * pow(fabs(c), 1 / 2.4) - 0.055);
    } else {
      OUT[i] = 12.92 * c;
    }
  }
}

const double LCH2OKLabMatrix[9] = {
  1, 0.3963377773761749, 0.2158037573099136,
  1, -0.1055613458156586, -0.0638541728258133,
  1, -0.0894841775298119, -1.2914855480194092
};

const double OKLab2XYZMatrix[9] = {
  1.2268798758459243, -0.5578149944602171, 0.2813910456659647,
  -0.0405757452148008, 1.1122868032803170, -0.0717110580655164,
  -0.0763729366746601, -0.4214933324022432, 1.5869240198367816
};

void oklab2xyz(const double *LAB, double *OUT) {
  double LMS[3];
  multiplyMatrices(LCH2OKLabMatrix, LAB, LMS);

  for (unsigned short i = 0; i < 3; i++) {
    LMS[i] = pow(LMS[i], 3);
  }

  multiplyMatrices(OKLab2XYZMatrix, LMS, OUT);
}

const double XYZ2LMSMatrix[9] = {
  0.8190224379967030, 0.3619062600528904, -0.1288737815209879,
  0.0329836539323885, 0.9292868615863434, 0.0361446663506424,
  0.0481771893596242, 0.2642395317527308, 0.6335478284694309
};

const double LMSg2OKLabMatrix[9] = {
  0.2104542683093140, 0.7936177747023054, -0.0040720430116193,
  1.9779985324311684, -2.4285922420485799, 0.4505937096174110,
  0.0259040424655478, 0.7827717124575296, -0.8086757549230774
};

void xyz2oklab(const double *XYZ, double *OUT) {
  double LMS[3];
  multiplyMatrices(XYZ2LMSMatrix, XYZ, LMS);

  double LMSg[3];
  for (unsigned short i = 0; i < 3; i++) {
    LMSg[i] = cbrt(LMS[i]);
  }

  multiplyMatrices(LMSg2OKLabMatrix, LMSg, OUT);
}

const double XYZ2RGBMatrix[9] = {
  3.2409699419045226, -1.537383177570094, -0.4986107602930034,
  -0.9692436362808796, 1.8759675015077202, 0.04155505740717559,
  0.05563007969699366, -0.20397695888897652, 1.0569715142428786
};

void xyz2rgbLinear(const double *XYZ, double *OUT) {
  multiplyMatrices(XYZ2RGBMatrix, XYZ, OUT);
}

const double RGB2XYZMatrix[9] = {
  0.41239079926595934, 0.357584339383878, 0.1804807884018343,
  0.21263900587151027, 0.715168678767756, 0.07219231536073371,
  0.01933081871559182, 0.11919477979462598, 0.9505321522496607
};

void rgbLinear2xyz(const double *RGB, double *OUT) {
  multiplyMatrices(RGB2XYZMatrix, RGB, OUT);
}

double clamp(double x, double minVal, double maxVal) {
  return fmax(minVal, fmin(x, maxVal));
}

void oklch2rgb(const double *LCH, int *OUT) {
  double OKLab[3];
  oklhch2oklab(LCH, OKLab);

  double XYZ[3];
  oklab2xyz(OKLab, XYZ);

  double RGBLinear[3];
  xyz2rgbLinear(XYZ, RGBLinear);

  double RGB[3];
  srgbLinear2rgb(RGBLinear, RGB);
  OUT[0] = (int)clamp((RGB[0] * (double)255), 0, 255);
  OUT[1] = (int)clamp((RGB[1] * (double)255), 0, 255);
  OUT[2] = (int)clamp((RGB[2] * (double)255), 0, 255);
}

void rgb2oklch(const int *RGB, double *OUT) {
  double RGBDouble[3];

  RGBDouble[0] = (double)RGB[0] / (double)255;
  RGBDouble[1] = (double)RGB[1] / (double)255;
  RGBDouble[2] = (double)RGB[2] / (double)255;

  double RGBLinear[3];
  rgb2srgbLinear(RGBDouble, RGBLinear);

  double XYZ[3];
  rgbLinear2xyz(RGBLinear, XYZ);

  double OKLab[3];
  xyz2oklab(XYZ, OKLab);

  oklab2oklch(OKLab, OUT);
}