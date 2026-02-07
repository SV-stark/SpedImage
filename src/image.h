#pragma once

#include "ImageBackend.h"
#include <cstdint>
#include <memory>
#include <string>
#include <vector>

class Image {
public:
  Image(const std::string &path);
  ~Image();

  bool Load();
  void Bind();
  void Unbind();

  // Memory Management
  // With WIC/Native, system memory might not be populated by default
  // (Zero-Copy) EnsureSystemMemory() downloads texture data from GPU if needed
  // for CPU processing
  void FreeSystemMemory();
  bool EnsureSystemMemory();

  // Modifiers (Used by Editor.cpp - uploads new data to texture)
  void SetData(const std::vector<uint8_t> &data, int width, int height,
               int channels);

  // Static helper for loading from memory buffer (fallback to stb_image
  // usually)
  static std::shared_ptr<Image> LoadFromMemory(const uint8_t *data,
                                               size_t size);

  // Accessors
  uint32_t GetTextureID() const;
  int GetWidth() const;
  int GetHeight() const;
  const std::vector<uint8_t> &GetData() const;
  int GetChannels() const { return 4; } // Normalized to RGBA

private:
  std::string m_Path;
  std::unique_ptr<ImageBackend> m_Backend;

  // Fallback state for manually set data (e.g. after editing)
  uint32_t m_ManualTextureID = 0;
  int m_ManualWidth = 0;
  int m_ManualHeight = 0;
  std::vector<uint8_t> m_Data;
};
