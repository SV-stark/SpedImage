#pragma once

class Image;

class Editor {
public:
  static void Crop(Image *img, int x, int y, int w, int h);
  static void Rotate(Image *img);
};
