#include "Editor.h"
#include "Image.h"
#include <GLFW/glfw3.h>
#include <cmath>
#include <iostream>
#include <vector>


// Using ImGui's loader, so we don't need GLAD
// Ensure imgui_impl_opengl3_loader.h is included in header or here
// It is in header, so we are good.

// Vertex Shader with Rotation and Crop/Zoom
const char *vertexShaderSource = R"(
#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

uniform float rotation; 
uniform float aspectRatio;
uniform vec4 cropRect; // x, y, w, h

void main() {
    gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);
    
    // 1. Crop/Zoom (Transform UVs to select sub-region)
    // Default: 0, 0, 1, 1 -> No change
    vec2 tex = aTexCoord * cropRect.zw + cropRect.xy;
    
    // 2. Rotation
    float s = sin(rotation);
    float c = cos(rotation);
    
    vec2 pos = tex - 0.5;
    pos.x *= aspectRatio;
    pos = mat2(c, -s, s, c) * pos;
    pos.x /= aspectRatio;
    TexCoord = pos + 0.5;
}
)";

// Fragment Shader with SCS (Saturation, Contrast, Brightness)
const char *fragmentShaderSource = R"(
#version 330 core
out vec4 FragColor;

in vec2 TexCoord;

uniform sampler2D imageTexture;
uniform float brightness;
uniform float contrast;
uniform float saturation;

void main() {
    vec4 texColor = texture(imageTexture, TexCoord);
    vec3 color = texColor.rgb;

    // Brightness
    color = color * brightness;

    // Contrast
    color = (color - 0.5) * contrast + 0.5;

    // Saturation
    float gray = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(vec3(gray), color, saturation);

    FragColor = vec4(color, texColor.a);
}
)";

Editor::Editor() {}

Editor::~Editor() {
  // Cleanup GL
  if (m_FBO)
    glDeleteFramebuffers(1, &m_FBO);
  if (m_TextureColorBuffer)
    glDeleteTextures(1, &m_TextureColorBuffer);
  if (m_RBO)
    glDeleteRenderbuffers(1, &m_RBO);
  if (m_ShaderProgram)
    glDeleteProgram(m_ShaderProgram);
  if (m_VAO)
    glDeleteVertexArrays(1, &m_VAO);
  if (m_VBO)
    glDeleteBuffers(1, &m_VBO);
}

void Editor::Init() {
  CreateShader();
  CreateQuad();
}

void Editor::CreateShader() {
  // 1. Vertex Shader
  unsigned int vertexShader = glCreateShader(GL_VERTEX_SHADER);
  glShaderSource(vertexShader, 1, &vertexShaderSource, NULL);
  glCompileShader(vertexShader);
  // Check errors
  int success;
  char infoLog[512];
  glGetShaderiv(vertexShader, GL_COMPILE_STATUS, &success);
  if (!success) {
    glGetShaderInfoLog(vertexShader, 512, NULL, infoLog);
    std::cerr << "ERROR::SHADER::VERTEX::COMPILATION_FAILED\n"
              << infoLog << std::endl;
  }

  // 2. Fragment Shader
  unsigned int fragmentShader = glCreateShader(GL_FRAGMENT_SHADER);
  glShaderSource(fragmentShader, 1, &fragmentShaderSource, NULL);
  glCompileShader(fragmentShader);
  // Check errors
  glGetShaderiv(fragmentShader, GL_COMPILE_STATUS, &success);
  if (!success) {
    glGetShaderInfoLog(fragmentShader, 512, NULL, infoLog);
    std::cerr << "ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n"
              << infoLog << std::endl;
  }

  // 3. Link Program
  m_ShaderProgram = glCreateProgram();
  glAttachShader(m_ShaderProgram, vertexShader);
  glAttachShader(m_ShaderProgram, fragmentShader);
  glLinkProgram(m_ShaderProgram);
  // Check errors
  glGetProgramiv(m_ShaderProgram, GL_LINK_STATUS, &success);
  if (!success) {
    glGetProgramInfoLog(m_ShaderProgram, 512, NULL, infoLog);
    std::cerr << "ERROR::SHADER::PROGRAM::LINKING_FAILED\n"
              << infoLog << std::endl;
  }

  glDeleteShader(vertexShader);
  glDeleteShader(fragmentShader);
}

void Editor::CreateQuad() {
  // Fullscreen quad
  float vertices[] = {
      // positions   // texCoords
      -1.0f, 1.0f,  0.0f, 1.0f, // Top-left
      -1.0f, -1.0f, 0.0f, 0.0f, // Bot-left
      1.0f,  -1.0f, 1.0f, 0.0f, // Bot-right

      -1.0f, 1.0f,  0.0f, 1.0f, // Top-left
      1.0f,  -1.0f, 1.0f, 0.0f, // Bot-right
      1.0f,  1.0f,  1.0f, 1.0f  // Top-right
  };

  glGenVertexArrays(1, &m_VAO);
  glGenBuffers(1, &m_VBO);

  glBindVertexArray(m_VAO);
  glBindBuffer(GL_ARRAY_BUFFER, m_VBO);
  glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

  // Pos
  glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 4 * sizeof(float), (void *)0);
  glEnableVertexAttribArray(0);
  // Tex
  glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 4 * sizeof(float),
                        (void *)(2 * sizeof(float)));
  glEnableVertexAttribArray(1);
}

void Editor::Resize(int width, int height) {
  if (m_Width == width && m_Height == height)
    return;
  if (width <= 0 || height <= 0)
    return;

  m_Width = width;
  m_Height = height;

  if (m_FBO) {
    glDeleteFramebuffers(1, &m_FBO);
    glDeleteTextures(1, &m_TextureColorBuffer);
    glDeleteRenderbuffers(1, &m_RBO);
  }

  glGenFramebuffers(1, &m_FBO);
  glBindFramebuffer(GL_FRAMEBUFFER, m_FBO);

  // Create texture
  glGenTextures(1, &m_TextureColorBuffer);
  glBindTexture(GL_TEXTURE_2D, m_TextureColorBuffer);
  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, width, height, 0, GL_RGBA,
               GL_UNSIGNED_BYTE, NULL);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D,
                         m_TextureColorBuffer, 0);

  if (glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE)
    std::cerr << "ERROR::FRAMEBUFFER:: Framebuffer is not complete!"
              << std::endl;

  glBindFramebuffer(GL_FRAMEBUFFER, 0);
}

void Editor::Render(std::shared_ptr<Image> img) {
  if (!img)
    return;
  if (m_ShaderProgram == 0)
    Init();

  // Ensure FBO matches image size
  Resize(img->GetWidth(), img->GetHeight());

  glBindFramebuffer(GL_FRAMEBUFFER, m_FBO);
  glViewport(0, 0, m_Width, m_Height);

  // Clear
  glClearColor(0.0f, 0.0f, 0.0f, 0.0f); // Transparent background
  glClear(GL_COLOR_BUFFER_BIT);

  // Shader Setup
  glUseProgram(m_ShaderProgram);
  glUniform1f(glGetUniformLocation(m_ShaderProgram, "brightness"), Brightness);
  glUniform1f(glGetUniformLocation(m_ShaderProgram, "contrast"), Contrast);
  glUniform1f(glGetUniformLocation(m_ShaderProgram, "saturation"), Saturation);

  glUniform1f(glGetUniformLocation(m_ShaderProgram, "rotation"), Rotation);

  float aspect = 1.0f;
  if (m_Height > 0)
    aspect = (float)m_Width / (float)m_Height;
  glUniform1f(glGetUniformLocation(m_ShaderProgram, "aspectRatio"), aspect);

  glUniform4f(glGetUniformLocation(m_ShaderProgram, "cropRect"), CropRect[0],
              CropRect[1], CropRect[2], CropRect[3]);

  // Bind Image Texture
  glActiveTexture(GL_TEXTURE0);
  glBindTexture(GL_TEXTURE_2D, img->GetTextureID());
  glUniform1i(glGetUniformLocation(m_ShaderProgram, "imageTexture"), 0);

  // Draw Quad
  glBindVertexArray(m_VAO);
  glDrawArrays(GL_TRIANGLES, 0, 6);

  glBindFramebuffer(GL_FRAMEBUFFER, 0);
}

void *Editor::GetTextureID() const {
  return (void *)(intptr_t)m_TextureColorBuffer;
}

std::vector<uint8_t> Editor::GetPixels() {
  if (m_Width <= 0 || m_Height <= 0)
    return {};

  std::vector<uint8_t> pixels(m_Width * m_Height * 4);

  if (m_FBO) {
    glBindFramebuffer(GL_FRAMEBUFFER, m_FBO);
    glReadPixels(0, 0, m_Width, m_Height, GL_RGBA, GL_UNSIGNED_BYTE,
                 pixels.data());
    glBindFramebuffer(GL_FRAMEBUFFER, 0);
  }

  return pixels;
}

// Legacy Stubs
void Editor::Crop(Image *img, int x, int y, int w, int h) {}
void Editor::RotateCPU(Image *img) {}
