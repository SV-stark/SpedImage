#if defined(__linux__) || defined(__unix__) || defined(__APPLE__)

#include "ImageBackend_Linux.h"
#include <algorithm>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>


// OpenGL
#include <GLFW/glfw3.h> // Includes GL headers

// Libs
#include <jpeglib.h>
#include <libheif/heif.h>
#include <spng.h>


ImageBackend_Linux::ImageBackend_Linux() {}

ImageBackend_Linux::~ImageBackend_Linux() { Release(); }

void ImageBackend_Linux::Release() {
  if (m_TextureID) {
    glDeleteTextures(1, &m_TextureID);
    m_TextureID = 0;
  }
}

bool ImageBackend_Linux::Load(const std::string &path) {
  std::string ext = std::filesystem::path(path).extension().string();
  std::transform(ext.begin(), ext.end(), ext.begin(), ::tolower);

  if (ext == ".jpg" || ext == ".jpeg") {
    return LoadJPEG(path);
  } else if (ext == ".png") {
    return LoadPNG(path);
  } else if (ext == ".heic" || ext == ".avif") {
    return LoadHeif(path);
  }

  // TODO: Add other formats (BMP, TGA, GIF) via stb_image fallback or other
  // libs? For now, these are the high-perf paths designated by plan.
  std::cerr << "Unsupported extension for Linux high-perf backend: " << ext
            << std::endl;
  return false;
}

bool ImageBackend_Linux::LoadJPEG(const std::string &path) {
  struct jpeg_decompress_struct cinfo;
  struct jpeg_error_mgr jerr;

  FILE *infile = fopen(path.c_str(), "rb");
  if (!infile) {
    std::cerr << "Can't open " << path << std::endl;
    return false;
  }

  cinfo.err = jpeg_std_error(&jerr);
  jpeg_create_decompress(&cinfo);
  jpeg_stdio_src(&cinfo, infile);
  jpeg_read_header(&cinfo, TRUE);

  // Force RGB
  cinfo.out_color_space = JCS_RGB;
  jpeg_start_decompress(&cinfo);

  m_Width = cinfo.output_width;
  m_Height = cinfo.output_height;
  int channels = cinfo.output_components; // Should be 3

  std::vector<uint8_t> buffer(m_Width * m_Height * channels);
  while (cinfo.output_scanline < cinfo.output_height) {
    uint8_t *row_pointer =
        &buffer[(cinfo.output_scanline) * m_Width * channels];
    jpeg_read_scanlines(&cinfo, &row_pointer, 1);
  }

  jpeg_finish_decompress(&cinfo);
  jpeg_destroy_decompress(&cinfo);
  fclose(infile);

  // Upload
  if (m_TextureID)
    glDeleteTextures(1, &m_TextureID);
  glGenTextures(1, &m_TextureID);
  glBindTexture(GL_TEXTURE_2D, m_TextureID);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S,
                  GL_CLAMP); // GL_CLAMP_TO_EDGE
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP);

  // JPEGs are usually RGB
  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, m_Width, m_Height, 0, GL_RGB,
               GL_UNSIGNED_BYTE, buffer.data());

  return true;
}

bool ImageBackend_Linux::LoadPNG(const std::string &path) {
  FILE *png = fopen(path.c_str(), "rb");
  if (!png)
    return false;

  spng_ctx *ctx = spng_ctx_new(0);
  spng_set_png_file(ctx, png);

  struct spng_ihdr ihdr;
  if (spng_get_ihdr(ctx, &ihdr)) {
    spng_ctx_free(ctx);
    fclose(png);
    return false;
  }

  m_Width = ihdr.width;
  m_Height = ihdr.height;

  size_t out_size;
  spng_decoded_image_size(ctx, SPNG_FMT_RGBA8, &out_size);
  std::vector<uint8_t> buffer(out_size);

  if (spng_decode_image(ctx, buffer.data(), out_size, SPNG_FMT_RGBA8, 0)) {
    spng_ctx_free(ctx);
    fclose(png);
    return false;
  }

  spng_ctx_free(ctx);
  fclose(png);

  // Upload
  if (m_TextureID)
    glDeleteTextures(1, &m_TextureID);
  glGenTextures(1, &m_TextureID);
  glBindTexture(GL_TEXTURE_2D, m_TextureID);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP);

  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, m_Width, m_Height, 0, GL_RGBA,
               GL_UNSIGNED_BYTE, buffer.data());
  return true;
}

bool ImageBackend_Linux::LoadHeif(const std::string &path) {
  heif_context *ctx = heif_context_alloc();
  if (!ctx)
    return false;

  heif_error err = heif_context_read_from_file(ctx, path.c_str(), nullptr);
  if (err.code != heif_error_Ok) {
    heif_context_free(ctx);
    return false;
  }

  heif_image_handle *handle = nullptr;
  err = heif_context_get_primary_image_handle(ctx, &handle);
  if (err.code != heif_error_Ok) {
    heif_context_free(ctx);
    return false;
  }

  heif_image *img = nullptr;
  err = heif_decode_image(handle, &img, heif_colorspace_RGB,
                          heif_chroma_interleaved_RGBA, nullptr);
  if (err.code != heif_error_Ok) {
    heif_image_handle_release(handle);
    heif_context_free(ctx);
    return false;
  }

  m_Width = heif_image_get_width(img, heif_channel_interleaved);
  m_Height = heif_image_get_height(img, heif_channel_interleaved);

  int stride;
  const uint8_t *data =
      heif_image_get_plane_readonly(img, heif_channel_interleaved, &stride);

  if (m_TextureID)
    glDeleteTextures(1, &m_TextureID);
  glGenTextures(1, &m_TextureID);
  glBindTexture(GL_TEXTURE_2D, m_TextureID);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP);

  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, m_Width, m_Height, 0, GL_RGBA,
               GL_UNSIGNED_BYTE, data);

  heif_image_release(img);
  heif_image_handle_release(handle);
  heif_context_free(ctx);
  return true;
}

bool ImageBackend_Linux::SupportsExtension(const std::string &ext) const {
  std::string lower = ext;
  std::transform(lower.begin(), lower.end(), lower.begin(), ::tolower);
  return (lower == ".jpg" || lower == ".jpeg" || lower == ".png" ||
          lower == ".heic" || lower == ".avif");
}

#endif
