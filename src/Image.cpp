#include "Image.h"
#include <iostream>
#include <memory>
#include <stb_image.h>
#include <stb_image_write.h>

#include <GLFW/glfw3.h>

Image::Image(const std::string &path) : m_Path(path) {}

Image::~Image() {
  if (m_RendererID) {
    glDeleteTextures(1, &m_RendererID);
    m_RendererID = 0;
  }
}

std::shared_ptr<Image> Image::LoadFromMemory(const uint8_t *data, size_t size) {
  auto img = std::make_shared<Image>("");
  int w, h, c;
  unsigned char *pixels = stbi_load_from_memory(data, (int)size, &w, &h, &c, 4);
  if (!pixels)
    return nullptr;

  img->SetData(std::vector<uint8_t>(pixels, pixels + (w * h * 4)), w, h, 4);
  stbi_image_free(pixels);
  img->FreeSystemMemory();
  return img;
}

void Image::SetData(const std::vector<uint8_t> &data, int width, int height,
                    int channels) {
  if (m_RendererID) {
    glDeleteTextures(1, &m_RendererID);
    m_RendererID = 0; // Reset m_RendererID after deletion
  }

  m_Width = width;
  m_Height = height;
  m_Channels = channels;
  m_Data = data;

  glGenTextures(1, &m_RendererID);
  glBindTexture(GL_TEXTURE_2D, m_RendererID);

  // Setup filtering parameters
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glTexParameteri(
      GL_TEXTURE_2D, GL_TEXTURE_WRAP_S,
      GL_CLAMP); // GL_CLAMP is deprecated but safe 1.1? GL_CLAMP_TO_EDGE
                 // is 1.2. On Windows 1.1 it's GL_CLAMP.
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP);

  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, width, height, 0, GL_RGBA,
               GL_UNSIGNED_BYTE, m_Data.data());
}

bool Image::Load() {
  int width, height, channels;
  stbi_set_flip_vertically_on_load(
      0); // Don't flip for 2D UI usually, or do if needed
  unsigned char *data =
      stbi_load(m_Path.c_str(), &width, &height, &channels, 4); // Force RGBA

  if (!data) {
    std::cerr << "Failed to load image: " << m_Path << std::endl;
    return false;
  }

  SetData(std::vector<uint8_t>(data, data + (width * height * 4)), width,
          height, 4);

  stbi_image_free(data);
  FreeSystemMemory();
  return true;
}

void Image::Bind() {
  if (m_RendererID)
    glBindTexture(GL_TEXTURE_2D, m_RendererID);
}

void Image::Unbind() { glBindTexture(GL_TEXTURE_2D, 0); }

void Image::FreeSystemMemory() {
  m_Data.clear();
  m_Data.shrink_to_fit();
}

bool Image::EnsureSystemMemory() {
  if (!m_Data.empty())
    return true;

  if (m_RendererID == 0)
    return false;

  // Assuming 4 channels as per Load/SetData usage
  size_t size = (size_t)m_Width * (size_t)m_Height * 4;
  m_Data.resize(size);

  glBindTexture(GL_TEXTURE_2D, m_RendererID);
  glGetTexImage(GL_TEXTURE_2D, 0, GL_RGBA, GL_UNSIGNED_BYTE, m_Data.data());
  return true;
}
