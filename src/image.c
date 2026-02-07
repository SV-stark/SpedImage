#include "image.h"
#include "utils.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "stb_image_write.h"

#define NANOSVG_IMPLEMENTATION
#include "nanosvg.h"

#define NANOSVGRAST_IMPLEMENTATION
#include "nanosvgrast.h"

static const char *get_extension(const char *filename) {
  const char *dot = strrchr(filename, '.');
  if (!dot || dot == filename)
    return "";
  return dot + 1;
}

ImageFormat image_detect_format(const char *filename) {
  ImageFormat format = IMG_FORMAT_UNKNOWN;
  const char *ext = get_extension(filename);

  if (strcasecmp(ext, "jpg") == 0 || strcasecmp(ext, "jpeg") == 0)
    format = IMG_FORMAT_JPEG;
  else if (strcasecmp(ext, "png") == 0)
    format = IMG_FORMAT_PNG;
  else if (strcasecmp(ext, "bmp") == 0)
    format = IMG_FORMAT_BMP;
  else if (strcasecmp(ext, "gif") == 0)
    format = IMG_FORMAT_GIF;
  else if (strcasecmp(ext, "tga") == 0)
    format = IMG_FORMAT_TGA;
  else if (strcasecmp(ext, "hdr") == 0)
    format = IMG_FORMAT_HDR;
  else if (strcasecmp(ext, "psd") == 0)
    format = IMG_FORMAT_PSD;
  else if (strcasecmp(ext, "pic") == 0)
    format = IMG_FORMAT_PIC;
  else if (strcasecmp(ext, "ppm") == 0 || strcasecmp(ext, "pgm") == 0 ||
           strcasecmp(ext, "pbm") == 0)
    format = IMG_FORMAT_PNM;
  else if (strcasecmp(ext, "svg") == 0)
    format = IMG_FORMAT_SVG;
  else if (strcasecmp(ext, "heic") == 0 || strcasecmp(ext, "heif") == 0)
    format = IMG_FORMAT_HEIC;
  else if (strcasecmp(ext, "avif") == 0)
    format = IMG_FORMAT_AVIF;
  else if (strcasecmp(ext, "webp") == 0)
    format = IMG_FORMAT_WEBP;
  else if (strcasecmp(ext, "tiff") == 0 || strcasecmp(ext, "tif") == 0)
    format = IMG_FORMAT_TIFF;
  else if (strcasecmp(ext, "raw") == 0 || strcasecmp(ext, "cr2") == 0 ||
           strcasecmp(ext, "nef") == 0 || strcasecmp(ext, "arw") == 0 ||
           strcasecmp(ext, "dng") == 0 || strcasecmp(ext, "orf") == 0 ||
           strcasecmp(ext, "raf") == 0 || strcasecmp(ext, "pef") == 0 ||
           strcasecmp(ext, "x3f") == 0) {
    format = IMG_FORMAT_RAW;
  }

  log_info("Detected format for %s: %d", filename, (int)format);
  return format;
}

static Image *image_from_svg(const char *filename, SDL_Renderer *renderer) {
  NSVGimage *svg = nsvgParseFromFile(filename, "px", 96.0f);
  if (!svg)
    return NULL;

  float scale = 1.0f;
  int w = (int)(svg->width * scale);
  int h = (int)(svg->height * scale);

  NSVGrasterizer *rast = nsvgCreateRasterizer();
  if (!rast) {
    nsvgDelete(svg);
    return NULL;
  }

  unsigned char *img_data = (unsigned char *)malloc(w * h * 4);
  if (!img_data) {
    nsvgDeleteRasterizer(rast);
    nsvgDelete(svg);
    return NULL;
  }

  nsvgRasterize(rast, svg, 0, 0, scale, img_data, w, h, w * 4);

  SDL_Surface *surface = SDL_CreateRGBSurfaceWithFormatFrom(
      img_data, w, h, 32, w * 4, SDL_PIXELFORMAT_RGBA32);

  Image *img = NULL;
  if (surface) {
    img = (Image *)calloc(1, sizeof(Image));
    if (img) {
      img->texture = SDL_CreateTextureFromSurface(renderer, surface);
      img->width = w;
      img->height = h;
      img->channels = 4;
      img->format = IMG_FORMAT_SVG;
      img->filename = strdup(filename);
      img->raw_data = img_data;
      img->raw_size = w * h * 4;
    }
    SDL_FreeSurface(surface);
  }

  nsvgDeleteRasterizer(rast);
  nsvgDelete(svg);

  return img;
}

Image *image_load(const char *filename, SDL_Renderer *renderer) {
  ImageFormat format = image_detect_format(filename);

  if (format == IMG_FORMAT_SVG) {
    return image_from_svg(filename, renderer);
  }

  int w, h, channels;
  unsigned char *data = NULL;

  // Try stb_image first
  if (format != IMG_FORMAT_HEIC && format != IMG_FORMAT_AVIF &&
      format != IMG_FORMAT_WEBP && format != IMG_FORMAT_TIFF &&
      format != IMG_FORMAT_RAW) {
    data = stbi_load(filename, &w, &h, &channels, 4);
  }

  if (!data) {
    fprintf(stderr, "Failed to load image: %s\n", filename);
    return NULL;
  }

  SDL_Surface *surface = SDL_CreateRGBSurfaceWithFormatFrom(
      data, w, h, 32, w * 4, SDL_PIXELFORMAT_RGBA32);

  if (!surface) {
    stbi_image_free(data);
    return NULL;
  }

  Image *img = (Image *)calloc(1, sizeof(Image));
  if (!img) {
    SDL_FreeSurface(surface);
    stbi_image_free(data);
    return NULL;
  }

  img->texture = SDL_CreateTextureFromSurface(renderer, surface);
  img->width = w;
  img->height = h;
  img->channels = channels;
  img->format = format;
  img->filename = strdup(filename);
  img->raw_data = data;
  img->raw_size = (size_t)w * h * 4;
  log_info("Image loaded successfully: %s (%dx%d)", filename, w, h);

  SDL_FreeSurface(surface);

  if (!img->texture) {
    image_free(img);
    return NULL;
  }

  return img;
}

void image_free(Image *img) {
  if (!img)
    return;

  if (img->texture) {
    SDL_DestroyTexture(img->texture);
  }
  if (img->filename) {
    free(img->filename);
  }
  if (img->raw_data) {
    if (img->format == IMG_FORMAT_SVG) {
      free(img->raw_data);
    } else {
      stbi_image_free(img->raw_data);
    }
  }

  free(img);
}

bool image_save(Image *img, const char *filename) {
  if (!img || !img->raw_data)
    return false;

  const char *ext = get_extension(filename);

  if (strcasecmp(ext, "png") == 0) {
    return stbi_write_png(filename, img->width, img->height, 4, img->raw_data,
                          img->width * 4) != 0;
  } else if (strcasecmp(ext, "jpg") == 0 || strcasecmp(ext, "jpeg") == 0) {
    return stbi_write_jpg(filename, img->width, img->height, 4, img->raw_data,
                          90) != 0;
  } else if (strcasecmp(ext, "bmp") == 0) {
    return stbi_write_bmp(filename, img->width, img->height, 4,
                          img->raw_data) != 0;
  } else if (strcasecmp(ext, "tga") == 0) {
    return stbi_write_tga(filename, img->width, img->height, 4,
                          img->raw_data) != 0;
  }

  return false;
}

void cache_init(ImageCache *cache, size_t max_memory) {
  cache->images = NULL;
  cache->count = 0;
  cache->capacity = 0;
  cache->current_index = -1;
  cache->max_memory = max_memory;
  cache->current_memory = 0;
}

void cache_cleanup(ImageCache *cache) {
  cache_clear(cache);
  free(cache->images);
  cache->images = NULL;
}

Image *cache_get(ImageCache *cache, int index) {
  if (index < 0 || index >= cache->count)
    return NULL;
  cache->current_index = index;
  return cache->images[index];
}

bool cache_add(ImageCache *cache, Image *img) {
  if (!img)
    return false;

  if (cache->count >= cache->capacity) {
    int new_capacity = cache->capacity == 0 ? 16 : cache->capacity * 2;
    Image **new_images = realloc(cache->images, new_capacity * sizeof(Image *));
    if (!new_images)
      return false;
    cache->images = new_images;
    cache->capacity = new_capacity;
  }

  cache->images[cache->count++] = img;
  cache->current_memory += img->raw_size;

  // Simple LRU: if memory exceeds max, remove oldest
  while (cache->current_memory > cache->max_memory && cache->count > 1) {
    if (cache->count > 0 && cache->images[0]) {
      cache->current_memory -= cache->images[0]->raw_size;
      image_free(cache->images[0]);
      memmove(&cache->images[0], &cache->images[1],
              (cache->count - 1) * sizeof(Image *));
      cache->count--;
      if (cache->current_index > 0)
        cache->current_index--;
    }
  }

  return true;
}

void cache_clear(ImageCache *cache) {
  for (int i = 0; i < cache->count; i++) {
    image_free(cache->images[i]);
  }
  cache->count = 0;
  cache->current_index = -1;
  cache->current_memory = 0;
}
