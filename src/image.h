#pragma once

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
  void FreeSystemMemory();
  bool EnsureSystemMemory();

  // Modifiers
  void SetData(const std::vector<uint8_t> &data, int width, int height,
               int channels);
  static std::shared_ptr<Image> LoadFromMemory(const uint8_t *data,
                                               size_t size);

  // Accessors
  uint32_t GetTextureID() const { return m_RendererID; }
  int GetWidth() const { return m_Width; }
  int GetHeight() const { return m_Height; }
  const std::vector<uint8_t> &GetData() const { return m_Data; }
  int GetChannels() const { return m_Channels; }

private:
  std::string m_Path;
  uint32_t m_RendererID = 0;
  int m_Width = 0;
  int m_Height = 0;
  int m_Channels = 0;
  std::vector<uint8_t> m_Data;
};
