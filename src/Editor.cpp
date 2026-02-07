#include "Editor.h"
#include "Image.h"
#include <algorithm>
#include <vector>

void Editor::Crop(Image *img, int x, int y, int w, int h) {
  if (!img)
    return;

  if (!img->EnsureSystemMemory())
    return;

  int srcW = img->GetWidth();
  int srcH = img->GetHeight();
  const auto &srcData = img->GetData();

  // Clamp
  if (x < 0)
    x = 0;
  if (y < 0)
    y = 0;
  if (x + w > srcW)
    w = srcW - x;
  if (y + h > srcH)
    h = srcH - y;
  if (w <= 0 || h <= 0)
    return;

  std::vector<uint8_t> newData(w * h * 4);

  for (int row = 0; row < h; row++) {
    const uint8_t *srcPtr = srcData.data() + ((y + row) * srcW + x) * 4;
    uint8_t *dstPtr = newData.data() + (row * w) * 4;
    std::copy(srcPtr, srcPtr + w * 4, dstPtr);
  }

  img->SetData(newData, w, h, 4);
  img->FreeSystemMemory();
}

void Editor::Rotate(Image *img) {
  // Rotate 90 degrees clockwise
  if (!img)
    return;

  if (!img->EnsureSystemMemory())
    return;

  int w = img->GetWidth();
  int h = img->GetHeight();
  const auto &srcData = img->GetData();

  std::vector<uint8_t> newData(w * h * 4);

  // 90 deg CW: (x, y) -> (h - 1 - y, x)
  // New width = h, New height = w

  int newW = h;
  int newH = w;

  for (int y = 0; y < h; y++) {
    for (int x = 0; x < w; x++) {
      // Source pixel
      int srcIdx = (y * w + x) * 4;

      // Dest pixel
      // dstX = h - 1 - y
      // dstY = x
      int dstX = h - 1 - y;
      int dstY = x;
      int dstIdx = (dstY * newW + dstX) * 4;

      newData[dstIdx + 0] = srcData[srcIdx + 0];
      newData[dstIdx + 1] = srcData[srcIdx + 1];
      newData[dstIdx + 2] = srcData[srcIdx + 2];
      newData[dstIdx + 3] = srcData[srcIdx + 3];
    }
  }

  img->SetData(newData, newW, newH, 4);
  img->FreeSystemMemory();
}
