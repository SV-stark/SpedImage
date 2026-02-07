#ifndef IMAGE_H
#define IMAGE_H

#include <SDL2/SDL.h>
#include <stdbool.h>
#include <stdint.h>

typedef enum {
  IMG_FORMAT_UNKNOWN,
  IMG_FORMAT_JPEG,
  IMG_FORMAT_PNG,
  IMG_FORMAT_BMP,
  IMG_FORMAT_GIF,
  IMG_FORMAT_TGA,
  IMG_FORMAT_HDR,
  IMG_FORMAT_PSD,
  IMG_FORMAT_PIC,
  IMG_FORMAT_PNM,
  IMG_FORMAT_SVG,
  IMG_FORMAT_HEIC,
  IMG_FORMAT_AVIF,
  IMG_FORMAT_WEBP,
  IMG_FORMAT_TIFF,
  IMG_FORMAT_RAW,
} ImageFormat;

typedef struct {
  SDL_Texture *texture;
  int width;
  int height;
  int channels;
  ImageFormat format;
  char *filename;
  uint8_t *raw_data;
  size_t raw_size;
} Image;

typedef struct {
  Image **images;
  int count;
  int capacity;
  int current_index;
  size_t max_memory;
  size_t current_memory;
} ImageCache;

ImageFormat image_detect_format(const char *filename);
Image *image_load(const char *filename, SDL_Renderer *renderer);
bool image_reload_raw(Image *img);
void image_free(Image *img);
bool image_save(Image *img, const char *filename);

void cache_init(ImageCache *cache, size_t max_memory);
void cache_cleanup(ImageCache *cache);
Image *cache_get(ImageCache *cache, int index);
bool cache_add(ImageCache *cache, Image *img);
void cache_clear(ImageCache *cache);

#endif
