#pragma once

#ifdef _WIN32

#include "ImageBackend.h"
#include <vector>
#include <wincodec.h>
#include <wrl/client.h>

using Microsoft::WRL::ComPtr;

class ImageBackend_Win32 : public ImageBackend {
public:
  ImageBackend_Win32();
  ~ImageBackend_Win32() override;

  bool Load(const std::string &path) override;
  void *GetTextureID() const override { return (void *)(intptr_t)m_TextureID; }
  int GetWidth() const override { return m_Width; }
  int GetHeight() const override { return m_Height; }
  void Release() override;
  bool SupportsExtension(const std::string &ext) const override;

private:
  bool CreateWICFactory();
  bool LoadWithLibHeif(const std::string &path);

  uint32_t m_TextureID = 0;
  int m_Width = 0;
  int m_Height = 0;

  // Static factory to avoid re-creating it for every image
  static ComPtr<IWICImagingFactory> s_WICFactory;
};

#endif // _WIN32
