#ifndef VIEWPORT_H
#define VIEWPORT_H

#include <SDL2/SDL.h>
#include "image.h"

typedef struct {
    float scale;
    float offset_x;
    float offset_y;
    int image_x;
    int image_y;
    int image_w;
    int image_h;
    bool fit_to_window;
} Viewport;

void viewport_init(Viewport* vp);
void viewport_reset(Viewport* vp);
void viewport_fit_image(Viewport* vp, int img_w, int img_h, int win_w, int win_h);
void viewport_zoom(Viewport* vp, float factor, int center_x, int center_y);
void viewport_pan(Viewport* vp, float dx, float dy);
void viewport_render(Viewport* vp, SDL_Renderer* renderer, Image* img);
SDL_Rect viewport_get_image_rect(Viewport* vp);

#endif
