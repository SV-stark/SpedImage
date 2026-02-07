#include "window.h"
#include <stdio.h>

bool window_init(AppWindow* win, const char* title) {
    if (SDL_Init(SDL_INIT_VIDEO) < 0) {
        fprintf(stderr, "SDL_Init failed: %s\n", SDL_GetError());
        return false;
    }
    
    win->width = WINDOW_DEFAULT_WIDTH;
    win->height = WINDOW_DEFAULT_HEIGHT;
    win->fullscreen = false;
    win->running = true;
    
    win->window = SDL_CreateWindow(
        title,
        SDL_WINDOWPOS_CENTERED,
        SDL_WINDOWPOS_CENTERED,
        win->width,
        win->height,
        SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE
    );
    
    if (!win->window) {
        fprintf(stderr, "SDL_CreateWindow failed: %s\n", SDL_GetError());
        SDL_Quit();
        return false;
    }
    
    win->renderer = SDL_CreateRenderer(
        win->window,
        -1,
        SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC
    );
    
    if (!win->renderer) {
        fprintf(stderr, "SDL_CreateRenderer failed: %s\n", SDL_GetError());
        SDL_DestroyWindow(win->window);
        SDL_Quit();
        return false;
    }
    
    SDL_SetWindowMinimumSize(win->window, WINDOW_MIN_WIDTH, WINDOW_MIN_HEIGHT);
    
    return true;
}

void window_cleanup(AppWindow* win) {
    if (win->renderer) {
        SDL_DestroyRenderer(win->renderer);
        win->renderer = NULL;
    }
    if (win->window) {
        SDL_DestroyWindow(win->window);
        win->window = NULL;
    }
    SDL_Quit();
}

void window_toggle_fullscreen(AppWindow* win) {
    win->fullscreen = !win->fullscreen;
    SDL_SetWindowFullscreen(
        win->window,
        win->fullscreen ? SDL_WINDOW_FULLSCREEN_DESKTOP : 0
    );
}

void window_get_size(AppWindow* win, int* w, int* h) {
    SDL_GetWindowSize(win->window, w, h);
    win->width = *w;
    win->height = *h;
}
