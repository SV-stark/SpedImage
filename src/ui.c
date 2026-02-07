#include "ui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

#define TOOLBAR_HEIGHT 50
#define TOOLBAR_BOTTOM_MARGIN 20
#define BUTTON_SIZE 40
#define BUTTON_PADDING 6
#define ICON_PADDING 10
#define AUTO_HIDE_DELAY 3000
#define SIDEBAR_WIDTH 200

// Helper to draw rounded rect
static void draw_rounded_rect(SDL_Renderer *renderer, SDL_Rect *rect,
                              int radius, SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);
  SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);

  // Central rects
  SDL_Rect r1 = {rect->x + radius, rect->y, rect->w - 2 * radius, rect->h};
  SDL_Rect r2 = {rect->x, rect->y + radius, rect->w, rect->h - 2 * radius};
  SDL_RenderFillRect(renderer, &r1);
  SDL_RenderFillRect(renderer, &r2);

  // Corners (simple approximation)
  // Top-left
  // (Removed unused rects bl, br, tr, tl)

  // Fill corners (for now just squares, fully rounded needs circle algo, but
  // this is fast) Let's do a better job - using points for corners? For
  // simplicity efficiently, we'll draw small rects to approximate circle or
  // just fill for now. Given "Modern", let's try to do actual points for best
  // look if radius is small.

  // Improved corner drawing
  for (int w = 0; w < radius; w++) {
    for (int h = 0; h < radius; h++) {
      if ((radius - w) * (radius - w) + (radius - h) * (radius - h) <=
          radius * radius) {
        SDL_RenderDrawPoint(renderer, rect->x + radius - w - 1,
                            rect->y + radius - h - 1); // TL
        SDL_RenderDrawPoint(renderer, rect->x + rect->w - radius + w,
                            rect->y + radius - h - 1); // TR
        SDL_RenderDrawPoint(renderer, rect->x + radius - w - 1,
                            rect->y + rect->h - radius + h); // BL
        SDL_RenderDrawPoint(renderer, rect->x + rect->w - radius + w,
                            rect->y + rect->h - radius + h); // BR
      }
    }
  }
}

// --- Icon Drawing Helpers ---

static void draw_icon_folder(SDL_Renderer *renderer, int x, int y, int size) {
  SDL_Rect body = {x + 2, y + 6, size - 4, size - 8};
  SDL_Rect tab = {x + 2, y + 2, size / 2 - 2, 4};
  SDL_RenderDrawRect(renderer, &body);
  SDL_RenderDrawRect(renderer, &tab);
  SDL_RenderDrawLine(
      renderer, x + 2, y + 6, x + size / 2,
      y + 6); // Clear line between tab and body? No, just draw structure.
}

static void draw_icon_floppy(SDL_Renderer *renderer, int x, int y, int size) {
  SDL_Rect body = {x + 4, y + 4, size - 8, size - 8};
  SDL_RenderDrawRect(renderer, &body);
  SDL_Rect inner = {x + 8, y + 4, size - 16, size / 2};
  SDL_RenderDrawRect(renderer, &inner);
}

static void draw_icon_arrow(SDL_Renderer *renderer, int x, int y, int size,
                            bool right) {
  int cx = x + size / 2;
  int cy = y + size / 2;
  int offset = size / 4;
  if (right) {
    SDL_RenderDrawLine(renderer, cx - offset, cy - offset, cx + offset, cy);
    SDL_RenderDrawLine(renderer, cx + offset, cy, cx - offset, cy + offset);
  } else {
    SDL_RenderDrawLine(renderer, cx + offset, cy - offset, cx - offset, cy);
    SDL_RenderDrawLine(renderer, cx - offset, cy, cx + offset, cy + offset);
  }
}

static void draw_icon_crop(SDL_Renderer *renderer, int x, int y, int size) {
  int m = 6;
  SDL_Rect r = {x + m, y + m, size - 2 * m, size - 2 * m};

  // Draw two intersecting corners
  // Top-left corner lines
  SDL_RenderDrawLine(renderer, r.x, r.y + r.h / 2, r.x, r.y);
  SDL_RenderDrawLine(renderer, r.x, r.y, r.x + r.w / 2, r.y);

  // Bottom-right corner lines
  SDL_RenderDrawLine(renderer, r.x + r.w, r.y + r.h / 2, r.x + r.w, r.y + r.h);
  SDL_RenderDrawLine(renderer, r.x + r.w / 2, r.y + r.h, r.x + r.w, r.y + r.h);

  // Diagnosis line
  SDL_RenderDrawLine(renderer, r.x, r.y + r.h, r.x + r.w, r.y);
}

static void draw_icon_rotate(SDL_Renderer *renderer, int x, int y, int size) {
  int cx = x + size / 2;
  int cy = y + size / 2;
  int r = size / 3;

  // Draw arc approximation (diamond)
  SDL_RenderDrawLine(renderer, cx + r, cy, cx, cy - r);
  SDL_RenderDrawLine(renderer, cx, cy - r, cx - r, cy);
  SDL_RenderDrawLine(renderer, cx - r, cy, cx, cy + r);

  // Arrow head at top
  SDL_RenderDrawLine(renderer, cx, cy - r, cx + 4, cy - r - 4);
  SDL_RenderDrawLine(renderer, cx, cy - r, cx + 4, cy - r + 4);
}

static void draw_icon_sun(SDL_Renderer *renderer, int x, int y, int size) {
  int cx = x + size / 2;
  int cy = y + size / 2;
  int r = size / 4;

  // Center circle
  SDL_Rect circ = {cx - r, cy - r, r * 2, r * 2};
  draw_rounded_rect(renderer, &circ, r, (SDL_Color){255, 255, 255, 255});

  // Rays
  SDL_RenderDrawLine(renderer, cx, y + 2, cx, y + 4);               // N
  SDL_RenderDrawLine(renderer, cx, y + size - 2, cx, y + size - 4); // S
  SDL_RenderDrawLine(renderer, x + 2, cy, x + 4, cy);               // W
  SDL_RenderDrawLine(renderer, x + size - 2, cy, x + size - 4, cy); // E
}

static void draw_icon_resize(SDL_Renderer *renderer, int x, int y, int size) {
  int m = 8;
  SDL_Rect r = {x + m, y + m, size - 2 * m, size - 2 * m};
  SDL_RenderDrawRect(renderer, &r);
  // Cross arrows
  SDL_RenderDrawLine(renderer, r.x, r.y, r.x + r.w, r.y + r.h);
  SDL_RenderDrawLine(renderer, r.x + r.w, r.y, r.x, r.y + r.h);
}

static void draw_icon_compress(SDL_Renderer *renderer, int x, int y, int size) {
  int m = 10;
  SDL_Rect r = {x + m, y + size / 2, size - 2 * m, size / 3};
  SDL_RenderDrawRect(renderer, &r);
  // Weight top
  SDL_RenderDrawLine(renderer, x + size / 2, y + m, x + size / 2, y + size / 2);
}

static void draw_icon_plus(SDL_Renderer *renderer, int x, int y, int size) {
  int cx = x + size / 2;
  int cy = y + size / 2;
  int len = size / 3;
  SDL_RenderDrawLine(renderer, cx - len, cy, cx + len, cy);
  SDL_RenderDrawLine(renderer, cx, cy - len, cx, cy + len);
}

static void draw_icon_minus(SDL_Renderer *renderer, int x, int y, int size) {
  int cx = x + size / 2;
  int cy = y + size / 2;
  int len = size / 3;
  SDL_RenderDrawLine(renderer, cx - len, cy, cx + len, cy);
}

static void draw_icon_fit(SDL_Renderer *renderer, int x, int y, int size) {
  int m = 8;
  // Four corners
  // TL
  SDL_RenderDrawLine(renderer, x + m, y + m, x + m + 4, y + m);
  SDL_RenderDrawLine(renderer, x + m, y + m, x + m, y + m + 4);
  // TR
  SDL_RenderDrawLine(renderer, x + size - m, y + m, x + size - m - 4, y + m);
  SDL_RenderDrawLine(renderer, x + size - m, y + m, x + size - m, y + m + 4);
  // BL
  SDL_RenderDrawLine(renderer, x + m, y + size - m, x + m + 4, y + size - m);
  SDL_RenderDrawLine(renderer, x + m, y + size - m, x + m, y + size - m - 4);
  // BR
  SDL_RenderDrawLine(renderer, x + size - m, y + size - m, x + size - m - 4,
                     y + size - m);
  SDL_RenderDrawLine(renderer, x + size - m, y + size - m, x + size - m,
                     y + size - m - 4);
}

static void draw_icon_fullscreen(SDL_Renderer *renderer, int x, int y,
                                 int size) {
  SDL_Rect r = {x + 8, y + 10, size - 16, size - 20};
  SDL_RenderDrawRect(renderer, &r);
  // Stand
  SDL_RenderDrawLine(renderer, x + size / 2, y + size - 10, x + size / 2,
                     y + size - 4);
  SDL_RenderDrawLine(renderer, x + size / 2 - 6, y + size - 4, x + size / 2 + 6,
                     y + size - 4);
}

static void draw_icon_quit(SDL_Renderer *renderer, int x, int y, int size) {
  int m = 10;
  SDL_RenderDrawLine(renderer, x + m, y + m, x + size - m, y + size - m);
  SDL_RenderDrawLine(renderer, x + size - m, y + m, x + m, y + size - m);
}

static void draw_icon(SDL_Renderer *renderer, int type, int x, int y,
                      int size) {
  SDL_SetRenderDrawColor(renderer, 220, 220, 220, 255);
  switch (type) {
  case 0:
    draw_icon_folder(renderer, x, y, size);
    break;
  case 1:
    draw_icon_floppy(renderer, x, y, size);
    break;
  case 2:
    draw_icon_arrow(renderer, x, y, size, false);
    break;
  case 3:
    draw_icon_arrow(renderer, x, y, size, true);
    break;
  case 4:
    draw_icon_crop(renderer, x, y, size);
    break;
  case 5:
    draw_icon_rotate(renderer, x, y, size);
    break;
  case 6:
    draw_icon_sun(renderer, x, y, size);
    break;
  case 7:
    draw_icon_resize(renderer, x, y, size);
    break;
  case 8:
    draw_icon_compress(renderer, x, y, size);
    break;
  case 9:
    draw_icon_plus(renderer, x, y, size);
    break;
  case 10:
    draw_icon_minus(renderer, x, y, size);
    break;
  case 11:
    draw_icon_fit(renderer, x, y, size);
    break;
  case 12:
    draw_icon_fullscreen(renderer, x, y, size);
    break;
  case 13:
    draw_icon_quit(renderer, x, y, size);
    break;
  }
}

// --- Main UI Functions ---

void ui_init(Toolbar *toolbar, Sidebar *sidebar) {
  toolbar->visible = true;
  toolbar->visible = true;
  toolbar->height = TOOLBAR_HEIGHT;
  toolbar->button_count = 14;
  toolbar->hovered_button = -1;
  toolbar->auto_hide = true;
  toolbar->last_activity = SDL_GetTicks();

  sidebar->visible = false;
  sidebar->width = SIDEBAR_WIDTH;
  sidebar->item_height = 100;
  sidebar->scroll_offset = 0;
  sidebar->item_count = 0;
}

void ui_cleanup(void) {
  // Nothing to clean up
}

UITool ui_handle_event(Toolbar *toolbar, Sidebar *sidebar, SDL_Event *event) {
  UITool result = UI_TOOL_NONE;

  if (event->type == SDL_MOUSEMOTION) {
    toolbar->last_activity = SDL_GetTicks();
    toolbar->visible = true;

    int mx = event->motion.x;
    int my = event->motion.y;

    // We need to know where the toolbar is.
    // Since we can't easily access window size here without passing it or
    // storing it, and we rely on rects stored in toolbar->buttons being up to
    // date.

    toolbar->hovered_button = -1;
    // Simple hit test against stored rects
    for (int i = 0; i < toolbar->button_count; i++) {
      if (mx >= toolbar->buttons[i].x &&
          mx < toolbar->buttons[i].x + toolbar->buttons[i].w &&
          my >= toolbar->buttons[i].y &&
          my < toolbar->buttons[i].y + toolbar->buttons[i].h) {
        toolbar->hovered_button = i;
        break;
      }
    }
  }

  if (event->type == SDL_MOUSEBUTTONDOWN &&
      event->button.button == SDL_BUTTON_LEFT) {
    if (toolbar->hovered_button >= 0) {
      // Mapping index to tool
      switch (toolbar->hovered_button) {
      case 0:
        result = UI_TOOL_OPEN;
        break;
      case 1:
        result = UI_TOOL_SAVE;
        break;
      case 2:
        result = UI_TOOL_PREV;
        break;
      case 3:
        result = UI_TOOL_NEXT;
        break;
      case 4:
        result = UI_TOOL_CROP;
        break;
      case 5:
        result = UI_TOOL_ROTATE;
        break;
      case 6:
        result = UI_TOOL_BRIGHTNESS;
        break;
      case 7:
        result = UI_TOOL_RESIZE;
        break;
      case 8:
        result = UI_TOOL_COMPRESS;
        break;
      case 9:
        result = UI_TOOL_ZOOM_IN;
        break;
      case 10:
        result = UI_TOOL_ZOOM_OUT;
        break;
      case 11:
        result = UI_TOOL_FIT;
        break;
      case 12:
        result = UI_TOOL_FULLSCREEN;
        break;
      case 13:
        result = UI_TOOL_QUIT;
        break;
      }
    }
  }

  if (toolbar->auto_hide && toolbar->visible) {
    Uint32 elapsed = SDL_GetTicks() - toolbar->last_activity;
    if (elapsed > AUTO_HIDE_DELAY) {
      int mx, my;
      SDL_GetMouseState(&mx, &my);

      bool hovering = false;
      for (int i = 0; i < toolbar->button_count; i++) {
        if (mx >= toolbar->buttons[i].x &&
            mx < toolbar->buttons[i].x + toolbar->buttons[i].w &&
            my >= toolbar->buttons[i].y &&
            my < toolbar->buttons[i].y + toolbar->buttons[i].h) {
          hovering = true;
          break;
        }
      }

      if (!hovering) {
        toolbar->visible = false;
      }
    }
  }

  return result;
}

void ui_render(Toolbar *toolbar, Sidebar *sidebar, SDL_Renderer *renderer) {
  int win_w, win_h;
  SDL_GetRendererOutputSize(renderer, &win_w, &win_h);

  // Render toolbar
  if (toolbar->visible) {
    int total_w =
        (BUTTON_SIZE + BUTTON_PADDING) * toolbar->button_count + BUTTON_PADDING;
    int start_x = (win_w - total_w) / 2;
    int start_y = win_h - TOOLBAR_HEIGHT - TOOLBAR_BOTTOM_MARGIN;

    // Draw Toolbar Capsule Background
    SDL_Rect toolbar_bg = {start_x, start_y, total_w, TOOLBAR_HEIGHT};
    draw_rounded_rect(renderer, &toolbar_bg, 10, (SDL_Color){30, 30, 30, 200});

    // Draw Border
    SDL_SetRenderDrawColor(renderer, 255, 255, 255, 30);
    SDL_RenderDrawRect(renderer, &toolbar_bg); // Simple rect border for now

    // Draw Buttons
    for (int i = 0; i < toolbar->button_count; i++) {
      SDL_Rect btn_rect = {start_x + BUTTON_PADDING +
                               i * (BUTTON_SIZE + BUTTON_PADDING),
                           start_y + (TOOLBAR_HEIGHT - BUTTON_SIZE) / 2,
                           BUTTON_SIZE, BUTTON_SIZE};

      // Update stored rect for hit testing
      toolbar->buttons[i] = btn_rect;

      bool hovered = (i == toolbar->hovered_button);

      // Background
      SDL_Color btn_color =
          hovered ? (SDL_Color){80, 80, 90, 255} : (SDL_Color){50, 50, 50, 0};
      if (hovered) {
        draw_rounded_rect(renderer, &btn_rect, 6, btn_color);
      }

      // Icon
      draw_icon(renderer, i, btn_rect.x, btn_rect.y, BUTTON_SIZE);
    }
  }

  // Render sidebar (kept simple for now)
  if (sidebar->visible) {
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 240);
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);
    SDL_Rect sidebar_rect = {0, 0, sidebar->width, win_h};
    SDL_RenderFillRect(renderer, &sidebar_rect);

    SDL_SetRenderDrawColor(renderer, 80, 80, 80, 255);
    SDL_RenderDrawLine(renderer, sidebar->width, 0, sidebar->width, win_h);
  }
}

void ui_show_toast(Toast *toast, const char *message, Uint32 duration) {
  strncpy(toast->message, message, sizeof(toast->message) - 1);
  toast->message[sizeof(toast->message) - 1] = '\0';
  toast->show_time = SDL_GetTicks();
  toast->duration = duration;
  toast->visible = true;
}

void ui_render_toast(Toast *toast, SDL_Renderer *renderer) {
  if (!toast->visible)
    return;

  Uint32 elapsed = SDL_GetTicks() - toast->show_time;
  if (elapsed > toast->duration) {
    toast->visible = false;
    return;
  }

  int win_w, win_h;
  SDL_GetRendererOutputSize(renderer, &win_w, &win_h);

  int padding = 15;
  int toast_w = strlen(toast->message) * 9 + padding * 2; // Approx char width
  int toast_h = 36;
  SDL_Rect toast_rect = {(win_w - toast_w) / 2,
                         win_h - 100, // Move up a bit
                         toast_w, toast_h};

  // Calculate alpha
  Uint8 alpha = 240;
  if (elapsed > toast->duration - 500) {
    alpha = (Uint8)(240 * (toast->duration - elapsed) / 500);
  }

  draw_rounded_rect(renderer, &toast_rect, 18, (SDL_Color){20, 20, 20, alpha});

  // Text approximation (since no fonts)
  // Draw simple lines for text or just rely on console/debug?
  // Wait, the original code didn't draw text either!
  // "In real app, render text with SDL_ttf" was in the comment.
  // The previous implementation dind't actually draw text!
  // So distinct toast messages were invisible except for the box size.

  // HACK: Draw "Toast" icon or dots to indicate message present.
  // Placeholder Icon / Dots logic already there
  SDL_SetRenderDrawColor(renderer, 255, 255, 255, alpha);
  SDL_Rect r = {toast_rect.x + 10, toast_rect.y + 16, toast_rect.w - 20, 2};
  SDL_RenderFillRect(renderer, &r);
}

void ui_render_placeholder(SDL_Renderer *renderer, int win_w, int win_h) {
  // Draw "Drop Image Here" placeholder
  SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);

  // Simple box
  int box_w = 300;
  int box_h = 200;
  SDL_Rect box = {(win_w - box_w) / 2, (win_h - box_h) / 2, box_w, box_h};
  draw_rounded_rect(renderer, &box, 20, (SDL_Color){40, 40, 40, 255});

  // Border
  SDL_SetRenderDrawColor(renderer, 80, 80, 80, 255);
  SDL_RenderDrawRect(renderer, &box);

  // "Icon" representation
  SDL_SetRenderDrawColor(renderer, 100, 100, 100, 255);
  int icon_size = 60;
  draw_icon_folder(renderer, (win_w - icon_size) / 2,
                   (win_h - icon_size) / 2 - 20, icon_size);

  // Text bars representation
  SDL_Rect bar1 = {box.x + 50, box.y + box_h - 60, box_w - 100, 4};
  SDL_Rect bar2 = {box.x + 80, box.y + box_h - 45, box_w - 160, 4};
  SDL_RenderFillRect(renderer, &bar1);
  SDL_RenderFillRect(renderer, &bar2);
}
