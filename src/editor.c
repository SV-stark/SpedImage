#include "editor.h"
#include <stdio.h>
#include <stdlib.h>
#include <math.h>

void editor_init(Editor* ed) {
    ed->mode = EDIT_MODE_NONE;
    ed->active = false;
    ed->crop_rect = (SDL_Rect){0, 0, 0, 0};
    ed->crop_dragging = false;
    ed->crop_drag_corner = -1;
    ed->rotation_angle = 0.0f;
    ed->brightness = 0.0f;
    ed->contrast = 1.0f;
    ed->original = NULL;
}

void editor_cleanup(Editor* ed) {
    editor_cancel(ed);
}

void editor_start_crop(Editor* ed) {
    ed->mode = EDIT_MODE_CROP;
    ed->active = true;
}

void editor_start_rotate(Editor* ed) {
    ed->mode = EDIT_MODE_ROTATE;
    ed->active = true;
}

void editor_start_brightness(Editor* ed) {
    ed->mode = EDIT_MODE_BRIGHTNESS;
    ed->active = true;
}

void editor_apply(Image* img, Editor* ed, SDL_Renderer* renderer) {
    if (!ed->active || !img) return;
    
    Image* new_img = NULL;
    
    switch (ed->mode) {
        case EDIT_MODE_CROP:
            if (ed->crop_rect.w > 0 && ed->crop_rect.h > 0) {
                new_img = image_crop(img, &ed->crop_rect, renderer);
            }
            break;
        case EDIT_MODE_ROTATE:
            new_img = image_rotate(img, ed->rotation_angle, renderer);
            break;
        case EDIT_MODE_BRIGHTNESS:
            new_img = image_adjust_brightness(img, ed->brightness, ed->contrast, renderer);
            break;
        default:
            break;
    }
    
    if (new_img) {
        // Replace texture and dimensions
        SDL_DestroyTexture(img->texture);
        free(img->raw_data);
        
        img->texture = new_img->texture;
        img->width = new_img->width;
        img->height = new_img->height;
        img->raw_data = new_img->raw_data;
        img->raw_size = new_img->raw_size;
        
        free(new_img);
    }
    
    editor_cancel(ed);
}

void editor_cancel(Editor* ed) {
    ed->mode = EDIT_MODE_NONE;
    ed->active = false;
    ed->crop_rect = (SDL_Rect){0, 0, 0, 0};
    ed->crop_dragging = false;
    ed->rotation_angle = 0.0f;
    ed->brightness = 0.0f;
    ed->contrast = 1.0f;
}

void editor_handle_event(Editor* ed, SDL_Event* event) {
    if (!ed->active) return;
    
    if (event->type == SDL_MOUSEBUTTONDOWN && ed->mode == EDIT_MODE_CROP) {
        // Start crop drag
    } else if (event->type == SDL_MOUSEMOTION && ed->crop_dragging) {
        // Update crop rect
    }
}

void editor_render_ui(Editor* ed, SDL_Renderer* renderer) {
    if (!ed->active) return;
    
    // Draw crop rectangle
    if (ed->mode == EDIT_MODE_CROP && ed->crop_rect.w > 0) {
        SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255);
        SDL_RenderDrawRect(renderer, &ed->crop_rect);
        
        // Draw corner handles
        const int handle_size = 8;
        for (int i = 0; i < 4; i++) {
            SDL_Rect handle = {
                (i % 2 == 0) ? ed->crop_rect.x : ed->crop_rect.x + ed->crop_rect.w - handle_size,
                (i < 2) ? ed->crop_rect.y : ed->crop_rect.y + ed->crop_rect.h - handle_size,
                handle_size,
                handle_size
            };
            SDL_RenderFillRect(renderer, &handle);
        }
    }
}

Image* image_crop(Image* src, SDL_Rect* rect, SDL_Renderer* renderer) {
    if (!src || !src->raw_data || !rect) return NULL;
    
    // Clamp rect to image bounds
    int x = rect->x < 0 ? 0 : rect->x;
    int y = rect->y < 0 ? 0 : rect->y;
    int w = rect->w;
    int h = rect->h;
    
    if (x + w > src->width) w = src->width - x;
    if (y + h > src->height) h = src->height - y;
    
    if (w <= 0 || h <= 0) return NULL;
    
    // Allocate new image data
    size_t new_size = w * h * 4;
    uint8_t* new_data = (uint8_t*)malloc(new_size);
    if (!new_data) return NULL;
    
    // Copy cropped region
    for (int row = 0; row < h; row++) {
        memcpy(
            new_data + row * w * 4,
            src->raw_data + ((y + row) * src->width + x) * 4,
            w * 4
        );
    }
    
    // Create new image
    Image* img = (Image*)calloc(1, sizeof(Image));
    if (!img) {
        free(new_data);
        return NULL;
    }
    
    SDL_Surface* surface = SDL_CreateRGBSurfaceWithFormatFrom(
        new_data, w, h, 32, w * 4, SDL_PIXELFORMAT_RGBA32
    );
    
    if (!surface) {
        free(new_data);
        free(img);
        return NULL;
    }
    
    img->texture = SDL_CreateTextureFromSurface(renderer, surface);
    img->width = w;
    img->height = h;
    img->channels = 4;
    img->format = src->format;
    img->filename = src->filename ? strdup(src->filename) : NULL;
    img->raw_data = new_data;
    img->raw_size = new_size;
    
    SDL_FreeSurface(surface);
    
    return img;
}

Image* image_rotate(Image* src, float angle, SDL_Renderer* renderer) {
    if (!src || !src->raw_data) return NULL;
    
    // Create target texture with rotation
    SDL_Texture* target = SDL_CreateTexture(
        renderer,
        SDL_PIXELFORMAT_RGBA32,
        SDL_TEXTUREACCESS_TARGET,
        src->width,
        src->height
    );
    
    if (!target) return NULL;
    
    // Set render target
    SDL_SetRenderTarget(renderer, target);
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 0);
    SDL_RenderClear(renderer);
    
    // Calculate center point
    SDL_Point center = {src->width / 2, src->height / 2};
    SDL_Rect dst_rect = {0, 0, src->width, src->height};
    
    // Render rotated
    SDL_RenderCopyEx(renderer, src->texture, NULL, &dst_rect, angle, &center, SDL_FLIP_NONE);
    
    // Read pixels back
    size_t size = src->width * src->height * 4;
    uint8_t* new_data = (uint8_t*)malloc(size);
    if (!new_data) {
        SDL_DestroyTexture(target);
        return NULL;
    }
    
    SDL_RenderReadPixels(renderer, NULL, SDL_PIXELFORMAT_RGBA32, new_data, src->width * 4);
    
    // Reset render target
    SDL_SetRenderTarget(renderer, NULL);
    
    // Create new image
    Image* img = (Image*)calloc(1, sizeof(Image));
    if (!img) {
        free(new_data);
        SDL_DestroyTexture(target);
        return NULL;
    }
    
    img->texture = target;
    img->width = src->width;
    img->height = src->height;
    img->channels = 4;
    img->format = src->format;
    img->filename = src->filename ? strdup(src->filename) : NULL;
    img->raw_data = new_data;
    img->raw_size = size;
    
    return img;
}

Image* image_adjust_brightness(Image* src, float brightness, float contrast, SDL_Renderer* renderer) {
    if (!src || !src->raw_data) return NULL;
    
    int w = src->width;
    int h = src->height;
    size_t size = w * h * 4;
    
    uint8_t* new_data = (uint8_t*)malloc(size);
    if (!new_data) return NULL;
    
    // Apply brightness and contrast
    for (size_t i = 0; i < size; i += 4) {
        for (int c = 0; c < 3; c++) { // RGB channels
            float val = src->raw_data[i + c];
            val = contrast * (val - 128) + 128 + brightness;
            if (val < 0) val = 0;
            if (val > 255) val = 255;
            new_data[i + c] = (uint8_t)val;
        }
        new_data[i + 3] = src->raw_data[i + 3]; // Alpha
    }
    
    // Create new image
    Image* img = (Image*)calloc(1, sizeof(Image));
    if (!img) {
        free(new_data);
        return NULL;
    }
    
    SDL_Surface* surface = SDL_CreateRGBSurfaceWithFormatFrom(
        new_data, w, h, 32, w * 4, SDL_PIXELFORMAT_RGBA32
    );
    
    if (!surface) {
        free(new_data);
        free(img);
        return NULL;
    }
    
    img->texture = SDL_CreateTextureFromSurface(renderer, surface);
    img->width = w;
    img->height = h;
    img->channels = 4;
    img->format = src->format;
    img->filename = src->filename ? strdup(src->filename) : NULL;
    img->raw_data = new_data;
    img->raw_size = size;
    
    SDL_FreeSurface(surface);
    
    return img;
}
