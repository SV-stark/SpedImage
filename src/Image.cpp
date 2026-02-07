#include "Image.h"
#include <iostream>
#include <vector>

// Backends
#ifdef _WIN32
#include "ImageBackend_Win32.h"
#elif defined(__linux__) || defined(__unix__) || defined(__APPLE__)
#include "ImageBackend_Linux.h"
#endif

// Fallback / Memory Load
#include <GLFW/glfw3.h> // For OpenGL
#include <stb_image.h>


Image::Image(const std::string &path) : m_Path(path) {}

Image::~Image() {
  if (m_ManualTextureID) {
    glDeleteTextures(1, &m_ManualTextureID);
  }
  // Backend handles its own cleanup
}

bool Image::Load() {
  // 1. Create Backend
#ifdef _WIN32
  m_Backend = std::make_unique<ImageBackend_Win32>();
#elif defined(__linux__) || defined(__unix__) || defined(__APPLE__)
  m_Backend = std::make_unique<ImageBackend_Linux>();
#else
  std::cerr << "Unsupported Platform for ImageBackend" << std::endl;
  return false;
#endif

  // 2. Try Load
  if (m_Backend && m_Backend->Load(m_Path)) {
    // Success
    return true;
  }

  // 3. Fallback to stb_image?
  // Only if backend failed or returned false (e.g. unknown extension)
  std::cerr << "Backend failed to load " << m_Path
            << ". Trying stb_image fallback..." << std::endl;

  int width, height, channels;
  stbi_set_flip_vertically_on_load(0);
  unsigned char *data =
      stbi_load(m_Path.c_str(), &width, &height, &channels, 4);

  if (data) {
    // Use manual path
    m_Backend.reset(); // Clear backend if it failed partially
    SetData(std::vector<uint8_t>(data, data + (width * height * 4)), width,
            height, 4);
    stbi_image_free(data);
    return true;
  }

  std::cerr << "Failed to load image: " << m_Path << std::endl;
  return false;
}

void Image::Bind() {
  uint32_t id = GetTextureID();
  if (id)
    glBindTexture(GL_TEXTURE_2D, id);
}

void Image::Unbind() { glBindTexture(GL_TEXTURE_2D, 0); }

uint32_t Image::GetTextureID() const {
  if (m_Backend)
    return (uint32_t)(uintptr_t)m_Backend->GetTextureID();
  return m_ManualTextureID;
}

int Image::GetWidth() const {
  if (m_Backend)
    return m_Backend->GetWidth();
  return m_ManualWidth;
}

int Image::GetHeight() const {
  if (m_Backend)
    return m_Backend->GetHeight();
  return m_ManualHeight;
}

const std::vector<uint8_t> &Image::GetData() const { return m_Data; }

void Image::FreeSystemMemory() {
  m_Data.clear();
  m_Data.shrink_to_fit();
}

bool Image::EnsureSystemMemory() {
  if (!m_Data.empty())
    return true;

  uint32_t id = GetTextureID();
  if (!id)
    return false;

  int w = GetWidth();
  int h = GetHeight();

  m_Data.resize(w * h * 4);

  // Download from GPU
  // Note: If using GL_BGRA upload, we should get back RGBA or BGRA?
  // glGetTexImage usually returns what we ask for. We want RGBA in CPU.
  glBindTexture(GL_TEXTURE_2D, id);
  glGetTexImage(GL_TEXTURE_2D, 0, GL_RGBA, GL_UNSIGNED_BYTE, m_Data.data());

  // Update manual dimensions so if we modify m_Data, we match
  m_ManualWidth = w;
  m_ManualHeight = h;

  return true;
}

void Image::SetData(const std::vector<uint8_t> &data, int width, int height,
                    int channels) {
  // If we are setting data manually, we invalidate the backend because the
  // image changed
  m_Backend.reset();

  if (m_ManualTextureID) {
    glDeleteTextures(1, &m_ManualTextureID);
    m_ManualTextureID = 0;
  }

  m_ManualWidth = width;
  m_ManualHeight = height;
  m_Data = data; // Copy

  glGenTextures(1, &m_ManualTextureID);
  glBindTexture(GL_TEXTURE_2D, m_ManualTextureID);

  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP);

  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, width, height, 0, GL_RGBA,
               GL_UNSIGNED_BYTE, m_Data.data());
}

std::shared_ptr<Image> Image::LoadFromMemory(const uint8_t *data, size_t size) {
  // STB fallback for memory loading (e.g. fonts/icons if used)
  auto img = std::make_shared<Image>("");
  int w, h, c;
  unsigned char *pixels = stbi_load_from_memory(data, (int)size, &w, &h, &c, 4);
  if (!pixels)
    return nullptr;

  img->SetData(std::vector<uint8_t>(pixels, pixels + (w * h * 4)), w, h, 4);
  stbi_image_free(pixels);
  return img;
}
