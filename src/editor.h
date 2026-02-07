#pragma once

#include <memory>
#include <vector>

// Use ImGui's built-in loader for convenience.
// If this fails, we might need adjustments based on include paths.
// But CMake adds backends/ to path, so <imgui_impl_opengl3_loader.h> should
// work.
#include <imgui_impl_opengl3_loader.h>

class Image;

class Editor {
public:
  Editor();
  ~Editor();

  void Init();

  // Renders the image with current adjustments to the internal FBO
  void Render(std::shared_ptr<Image> img);

  // Returns the texture ID of the FBO (for ImGui to display)
  void *GetTextureID() const;

  // Read back pixels from FBO (RGBA)
  std::vector<uint8_t> GetPixels();

  // Resize the FBO if the viewport changes (or to match image size)
  void Resize(int width, int height);

  // Adjustments
  float Brightness = 1.0f;
  float Contrast = 1.0f;
  float Saturation = 1.0f;
  float Rotation = 0.0f;                        // Radians
  float CropRect[4] = {0.0f, 0.0f, 1.0f, 1.0f}; // x, y, w, h (Normalized UVs)

  // Destructive CPU operations (legacy support or final apply)
  static void Crop(Image *img, int x, int y, int w, int h);
  static void RotateCPU(Image *img); // Renamed from Rotate

private:
  unsigned int m_FBO = 0;
  unsigned int m_TextureColorBuffer = 0;
  unsigned int m_RBO = 0;
  int m_Width = 0;
  int m_Height = 0;

  // Shader
  unsigned int m_ShaderProgram = 0;
  unsigned int m_VAO = 0;
  unsigned int m_VBO = 0;

  void CreateShader();
  void CreateQuad();
};
