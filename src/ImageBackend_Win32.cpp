#ifdef _WIN32

#include "ImageBackend_Win32.h"
#include <GLFW/glfw3.h> // For OpenGL headers
#include <algorithm>
#include <filesystem>
#include <iostream>
#include <libheif/heif.h>
#include <shlwapi.h>

#pragma comment(lib, "windowscodecs.lib")
#pragma comment(lib, "shlwapi.lib")

ComPtr<IWICImagingFactory> ImageBackend_Win32::s_WICFactory = nullptr;

ImageBackend_Win32::ImageBackend_Win32() { CreateWICFactory(); }

ImageBackend_Win32::~ImageBackend_Win32() { Release(); }

void ImageBackend_Win32::Release() {
  if (m_TextureID) {
    glDeleteTextures(1, &m_TextureID);
    m_TextureID = 0;
  }
}

bool ImageBackend_Win32::CreateWICFactory() {
  if (s_WICFactory)
    return true;

  HRESULT hr =
      CoCreateInstance(CLSID_WICImagingFactory, nullptr, CLSCTX_INPROC_SERVER,
                       IID_PPV_ARGS(&s_WICFactory));

  if (FAILED(hr)) {
    std::cerr << "Failed to create WIC Factory: " << std::hex << hr
              << std::endl;
    return false;
  }
  return true;
}

bool ImageBackend_Win32::Load(const std::string &path) {
  if (!s_WICFactory && !CreateWICFactory())
    return false;

  // Convert path to wide string
  std::wstring wpath(path.begin(), path.end());

  ComPtr<IWICBitmapDecoder> decoder;
  HRESULT hr = s_WICFactory->CreateDecoderFromFilename(
      wpath.c_str(), nullptr, GENERIC_READ, WICDecodeMetadataCacheOnDemand,
      &decoder);

  // Fallback: Check if file exists, if so, maybe codec is missing
  if (FAILED(hr)) {
    // Fallback: Check if file exists, if so, maybe codec is missing
    std::string ext = std::filesystem::path(path).extension().string();
    std::transform(ext.begin(), ext.end(), ext.begin(), ::tolower);

    if (ext == ".heic" || ext == ".avif") {
      if (LoadWithLibHeif(path))
        return true;
    }

    std::string msg = "Failed to load image: " + path +
                      "\nIt seems you are missing a codec (e.g. HEIC/Raw "
                      "Extension) or the file is corrupted.\nPlease install "
                      "the required extension from the Microsoft Store.";
    MessageBoxA(NULL, msg.c_str(), "SpedImage Error", MB_OK | MB_ICONERROR);
    std::cerr << "WIC Failed to load: " << path << " (HR: " << std::hex << hr
              << ")" << std::endl;
    return false;
  }

  ComPtr<IWICBitmapFrameDecode> frame;
  hr = decoder->GetFrame(0, &frame);
  if (FAILED(hr))
    return false;

  UINT w, h;
  frame->GetSize(&w, &h);
  m_Width = (int)w;
  m_Height = (int)h;

  // Convert to BGRA (compatible with OpenGL GL_BGRA)
  ComPtr<IWICFormatConverter> converter;
  s_WICFactory->CreateFormatConverter(&converter);

  // We use PBGRA (Premultiplied) or BGRA. OpenGL standard blending usually
  // expects non-premultiplied if using GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA But
  // WIC often handles PBGRA better. Let's try regular BGRA first.
  hr = converter->Initialize(frame.Get(), GUID_WICPixelFormat32bppBGRA,
                             WICBitmapDitherTypeNone, nullptr, 0.0,
                             WICBitmapPaletteTypeCustom);

  if (FAILED(hr))
    return false;

  // Allocate buffer
  // TODO: Direct upload via PBO to avoid this allocation for "Zero Copy"
  std::vector<uint8_t> buffer(m_Width * m_Height * 4);
  hr = converter->CopyPixels(nullptr, m_Width * 4, (UINT)buffer.size(),
                             buffer.data());

  if (FAILED(hr))
    return false;

  // OpenGL Upload
  if (m_TextureID)
    glDeleteTextures(1, &m_TextureID);

  glGenTextures(1, &m_TextureID);
  glBindTexture(GL_TEXTURE_2D, m_TextureID);

  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP);

  // Use GL_BGRA for the format because WIC gives us BGRA
  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, m_Width, m_Height, 0, GL_BGRA,
               GL_UNSIGNED_BYTE, buffer.data());

  return true;
}

bool ImageBackend_Win32::SupportsExtension(const std::string &ext) const {
  // WIC supports almost everything if codec is installed
  return true;
}

#endif

bool ImageBackend_Win32::LoadWithLibHeif(const std::string &path) {
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

  int w = heif_image_get_width(img, heif_channel_interleaved);
  int h = heif_image_get_height(img, heif_channel_interleaved);
  m_Width = w;
  m_Height = h;

  int stride;
  const uint8_t *data =
      heif_image_get_plane_readonly(img, heif_channel_interleaved, &stride);

  // OpenGL Upload
  if (m_TextureID)
    glDeleteTextures(1, &m_TextureID);

  glGenTextures(1, &m_TextureID);
  glBindTexture(GL_TEXTURE_2D, m_TextureID);

  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP);

  // libheif gives RGBA (as requested), so use GL_RGBA
  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, w, h, 0, GL_RGBA, GL_UNSIGNED_BYTE,
               data);

  heif_image_release(img);
  heif_image_handle_release(handle);
  heif_context_free(ctx);

  return true;
}
