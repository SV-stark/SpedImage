#pragma once

#include <string>
#include <cstdint>

/**
 * @brief Abstract interface for platform-specific image loading backends.
 * 
 * Implementations (WIC for Windows, LibJpeg/LibSpng for Linux) will handle
 * decoding images directly to OpenGL textures for maximum performance.
 */
class ImageBackend {
public:
    virtual ~ImageBackend() = default;

    /**
     * @brief Load an image from the filesystem.
     * @param path Absolute path to the image file.
     * @return true if loaded successfully, false otherwise.
     */
    virtual bool Load(const std::string& path) = 0;

    /**
     * @brief Get the OpenGL Texture ID.
     * @return GLuint texture ID.
     */
    virtual void* GetTextureID() const = 0;

    /**
     * @brief Get the width of the loaded image.
     */
    virtual int GetWidth() const = 0;

    /**
     * @brief Get the height of the loaded image.
     */
    virtual int GetHeight() const = 0;

    /**
     * @brief Releases OpenGL resources.
     */
    virtual void Release() = 0;
    
    /**
     * @brief Check if the backend supports the given file extension.
     * Useful for early-out or choosing between multiple backends.
     */
    virtual bool SupportsExtension(const std::string& ext) const = 0;
};
