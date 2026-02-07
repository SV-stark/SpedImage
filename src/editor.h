#ifndef EDITOR_H
#define EDITOR_H

#include <SDL2/SDL.h>
#include "image.h"

typedef enum {
    EDIT_MODE_NONE,
    EDIT_MODE_CROP,
    EDIT_MODE_ROTATE,
    EDIT_MODE_BRIGHTNESS,
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
    
    // Original backup
    Image* original;
} Editor;

void editor_init(Editor* ed);
void editor_cleanup(Editor* ed);
void editor_start_crop(Editor* ed);
void editor_start_rotate(Editor* ed);
void editor_start_brightness(Editor* ed);
void editor_apply(Image* img, Editor* ed, SDL_Renderer* renderer);
void editor_cancel(Editor* ed);
void editor_render_ui(Editor* ed, SDL_Renderer* renderer);
void editor_handle_event(Editor* ed, SDL_Event* event);

Image* image_crop(Image* src, SDL_Rect* rect, SDL_Renderer* renderer);
Image* image_rotate(Image* src, float angle, SDL_Renderer* renderer);
Image* image_adjust_brightness(Image* src, float brightness, float contrast, SDL_Renderer* renderer);

#endif
