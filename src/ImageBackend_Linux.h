#pragma once

#if defined(__linux__) || defined(__unix__) || defined(__APPLE__)

#include "ImageBackend.h"
#include <vector>

class ImageBackend_Linux : public ImageBackend {
public:
  ImageBackend_Linux();
  ~ImageBackend_Linux() override;

  bool Load(const std::string &path) override;
  void *GetTextureID() const override { return (void *)(intptr_t)m_TextureID; }
  int GetWidth() const override { return m_Width; }
  int GetHeight() const override { return m_Height; }
  void Release() override;
  bool SupportsExtension(const std::string &ext) const override;

private:
  bool LoadJPEG(const std::string &path);
  bool LoadPNG(const std::string &path);
  bool LoadHeif(const std::string &path);

  uint32_t m_TextureID = 0;
  int m_Width = 0;
  int m_Height = 0;
};

#endif
