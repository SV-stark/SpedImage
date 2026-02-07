#ifndef WINDOW_H
#define WINDOW_H

#include <SDL2/SDL.h>
#include <stdbool.h>

#define WINDOW_MIN_WIDTH 400
#define WINDOW_MIN_HEIGHT 300
#define WINDOW_DEFAULT_WIDTH 1024
#define WINDOW_DEFAULT_HEIGHT 768

typedef struct {
    SDL_Window* window;
    SDL_Renderer* renderer;
    int width;
    int height;
    bool fullscreen;
    bool running;
} AppWindow;

bool window_init(AppWindow* win, const char* title);
void window_cleanup(AppWindow* win);
void window_toggle_fullscreen(AppWindow* win);
void window_get_size(AppWindow* win, int* w, int* h);

#endif
