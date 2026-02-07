#ifndef UI_H
#define UI_H

#include <SDL2/SDL.h>
#include <stdbool.h>

typedef enum {
    UI_TOOL_NONE,
    UI_TOOL_OPEN,
    UI_TOOL_SAVE,
    UI_TOOL_PREV,
    UI_TOOL_NEXT,
    UI_TOOL_CROP,
    UI_TOOL_ROTATE,
    UI_TOOL_BRIGHTNESS,
    UI_TOOL_ZOOM_IN,
    UI_TOOL_ZOOM_OUT,
    UI_TOOL_FIT,
    UI_TOOL_FULLSCREEN,
    UI_TOOL_SLIDESHOW,
    UI_TOOL_INFO,
    UI_TOOL_QUIT,
} UITool;

typedef struct {
    bool visible;
    int height;
    SDL_Rect buttons[16];
    int button_count;
    int hovered_button;
    bool auto_hide;
    Uint32 last_activity;
} Toolbar;

typedef struct {
    bool visible;
    int width;
    int item_height;
    int scroll_offset;
    SDL_Rect items[100];
    int item_count;
} Sidebar;

typedef struct {
    bool visible;
    char message[256];
    Uint32 show_time;
    Uint32 duration;
} Toast;

void ui_init(Toolbar* toolbar, Sidebar* sidebar);
void ui_cleanup(void);
UITool ui_handle_event(Toolbar* toolbar, Sidebar* sidebar, SDL_Event* event);
void ui_render(Toolbar* toolbar, Sidebar* sidebar, SDL_Renderer* renderer);
void ui_show_toast(Toast* toast, const char* message, Uint32 duration);
void ui_render_toast(Toast* toast, SDL_Renderer* renderer);

#endif
