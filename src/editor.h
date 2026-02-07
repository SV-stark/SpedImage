#ifndef EDITOR_H
#define EDITOR_H

#include "image.h"
#include <SDL2/SDL.h>

typedef enum {
  EDIT_MODE_NONE,
  EDIT_MODE_CROP,
  EDIT_MODE_ROTATE,
  EDIT_MODE_BRIGHTNESS,
  EDIT_MODE_RESIZE,
  EDIT_MODE_COMPRESS,
} EditMode;

typedef struct {
  EditMode mode;
  bool active;

  // Crop
  SDL_Rect crop_rect;
  bool crop_dragging;
  int crop_drag_corner;

  // Rotation
  float rotation_angle;

  // Brightness/Contrast
  float brightness;
  float contrast;

  // Resize
  int target_width;
  int target_height;
  bool maintain_aspect;

  // Compression
  int target_size_kb;
  int calculated_quality; // Resulting JPEG quality

  // Original backup
  Image *original;
} Editor; // Editor operations
void editor_rotate_left(Editor *editor);
void editor_rotate_right(Editor *editor);
void editor_crop(Editor *ed);
void editor_init(Editor *ed);
void editor_cleanup(Editor *ed);
void editor_start_crop(Editor *ed);
void editor_start_rotate(Editor *ed);
void editor_start_brightness(Editor *ed);
void editor_apply(Image *img, Editor *ed, SDL_Renderer *renderer);
void editor_cancel(Editor *ed);
void editor_render_ui(Editor *ed, SDL_Renderer *renderer);
void editor_handle_event(Editor *ed, SDL_Event *event);

Image *image_crop(Image *src, SDL_Rect *rect, SDL_Renderer *renderer);
Image *image_rotate(Image *src, float angle, SDL_Renderer *renderer);
Image *image_adjust_brightness(Image *src, float brightness, float contrast,
                               SDL_Renderer *renderer);
Image *image_resize(Image *src, int target_w, int target_h,
                    SDL_Renderer *renderer);
Image *image_compress_preview(Image *src, int target_size_kb, int *out_quality,
                              SDL_Renderer *renderer);

void editor_start_resize(Editor *ed, int current_w, int current_h);
void editor_start_compress(Editor *ed, size_t current_size);

#endif
