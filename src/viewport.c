#include "viewport.h"
#include <math.h>

void viewport_init(Viewport* vp) {
    vp->scale = 1.0f;
    vp->offset_x = 0;
    vp->offset_y = 0;
    vp->image_x = 0;
    vp->image_y = 0;
    vp->image_w = 0;
    vp->image_h = 0;
    vp->fit_to_window = true;
}

void viewport_reset(Viewport* vp) {
    viewport_init(vp);
}

void viewport_fit_image(Viewport* vp, int img_w, int img_h, int win_w, int win_h) {
    if (img_w <= 0 || img_h <= 0) return;
    
    float scale_x = (float)win_w / img_w;
    float scale_y = (float)win_h / img_h;
    vp->scale = fminf(scale_x, scale_y);
    
    // Don't upscale small images too much
    if (vp->scale > 1.0f) {
        vp->scale = 1.0f;
    }
    
    vp->image_w = (int)(img_w * vp->scale);
    vp->image_h = (int)(img_h * vp->scale);
    vp->image_x = (win_w - vp->image_w) / 2;
    vp->image_y = (win_h - vp->image_h) / 2;
    vp->offset_x = 0;
    vp->offset_y = 0;
}

void viewport_zoom(Viewport* vp, float factor, int center_x, int center_y) {
    float old_scale = vp->scale;
    vp->scale *= factor;
    
    // Limit zoom levels
    if (vp->scale < 0.1f) vp->scale = 0.1f;
    if (vp->scale > 10.0f) vp->scale = 10.0f;
    
    // Zoom toward center point
    float scale_change = vp->scale / old_scale;
    vp->image_x = center_x - (center_x - vp->image_x) * scale_change;
    vp->image_y = center_y - (center_y - vp->image_y) * scale_change;
    vp->image_w = (int)(vp->image_w * scale_change);
    vp->image_h = (int)(vp->image_h * scale_change);
    
    vp->fit_to_window = false;
}

void viewport_pan(Viewport* vp, float dx, float dy) {
    vp->image_x += dx;
    vp->image_y += dy;
    vp->offset_x += dx;
    vp->offset_y += dy;
    vp->fit_to_window = false;
}

void viewport_render(Viewport* vp, SDL_Renderer* renderer, Image* img) {
    if (!img || !img->texture) return;
    
    SDL_Rect dst_rect = {
        vp->image_x,
        vp->image_y,
        vp->image_w,
        vp->image_h
    };
    
    SDL_RenderCopy(renderer, img->texture, NULL, &dst_rect);
}

SDL_Rect viewport_get_image_rect(Viewport* vp) {
    SDL_Rect rect = {
        vp->image_x,
        vp->image_y,
        vp->image_w,
        vp->image_h
    };
    return rect;
}
