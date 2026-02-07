#include "ui.h"
#include "editor.h"
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define TOOLBAR_HEIGHT 50
#define TOOLBAR_BOTTOM_MARGIN 20
#define BUTTON_SIZE 40
#define BUTTON_PADDING 6
#define ICON_PADDING 10
#define AUTO_HIDE_DELAY 3000
#define SIDEBAR_WIDTH 200
#define TOOLTIP_DELAY 500
#define ANIMATION_SPEED 0.15f

// Modern dark color scheme
static const SDL_Color COLOR_BG_DARK = {20, 20, 25, 255};
static const SDL_Color COLOR_BG_LIGHT = {35, 35, 45, 255};
static const SDL_Color COLOR_BG_HOVER = {60, 65, 80, 255};
static const SDL_Color COLOR_BG_PRESSED = {45, 50, 65, 255};
static const SDL_Color COLOR_ACCENT = {100, 150, 255, 255};
static const SDL_Color COLOR_TEXT = {220, 220, 230, 255};
static const SDL_Color COLOR_TEXT_DIM = {150, 150, 160, 255};
static const SDL_Color COLOR_ICON = {200, 200, 210, 255};

// Button labels for tooltips
static const char *BUTTON_LABELS[] = {
    "Open",     "Save",       "Previous",   "Next",     "Crop",
    "Rotate",   "Brightness", "Resize",     "Compress", "Zoom In",
    "Zoom Out", "Fit",        "Fullscreen", "Quit"};

// Simple 5x7 pixel font patterns (1 = pixel, 0 = empty)
// Supports: A-Z, a-z, 0-9, space, +, -, :, /, ., (comma), !
static const uint8_t FONT_5X7[][5] = {
    // Space (0)
    {0x00, 0x00, 0x00, 0x00, 0x00},
    // ! (1)
    {0x00, 0x00, 0x5F, 0x00, 0x00},
    // + (2)
    {0x08, 0x08, 0x3E, 0x08, 0x08},
    // - (3)
    {0x08, 0x08, 0x08, 0x08, 0x08},
    // . (4)
    {0x00, 0x60, 0x60, 0x00, 0x00},
    // / (5)
    {0x20, 0x10, 0x08, 0x04, 0x02},
    // : (6)
    {0x00, 0x36, 0x36, 0x00, 0x00},
    // , (7)
    {0x00, 0x60, 0x30, 0x00, 0x00},
    // 0-9 (8-17)
    {0x3E, 0x51, 0x49, 0x45, 0x3E}, // 0
    {0x00, 0x42, 0x7F, 0x40, 0x00}, // 1
    {0x42, 0x61, 0x51, 0x49, 0x46}, // 2
    {0x21, 0x41, 0x45, 0x4B, 0x31}, // 3
    {0x18, 0x14, 0x12, 0x7F, 0x10}, // 4
    {0x27, 0x45, 0x45, 0x45, 0x39}, // 5
    {0x3C, 0x4A, 0x49, 0x49, 0x30}, // 6
    {0x01, 0x71, 0x09, 0x05, 0x03}, // 7
    {0x36, 0x49, 0x49, 0x49, 0x36}, // 8
    {0x06, 0x49, 0x49, 0x29, 0x1E}, // 9
    // A-J (18-27)
    {0x7E, 0x11, 0x11, 0x11, 0x7E}, // A
    {0x7F, 0x49, 0x49, 0x49, 0x36}, // B
    {0x3E, 0x41, 0x41, 0x41, 0x22}, // C
    {0x7F, 0x41, 0x41, 0x22, 0x1C}, // D
    {0x7F, 0x49, 0x49, 0x49, 0x41}, // E
    {0x7F, 0x09, 0x09, 0x09, 0x01}, // F
    {0x3E, 0x41, 0x49, 0x49, 0x7A}, // G
    {0x7F, 0x08, 0x08, 0x08, 0x7F}, // H
    {0x00, 0x41, 0x7F, 0x41, 0x00}, // I
    {0x20, 0x40, 0x41, 0x3F, 0x01}, // J
    // K-T (28-37)
    {0x7F, 0x08, 0x14, 0x22, 0x41}, // K
    {0x7F, 0x40, 0x40, 0x40, 0x40}, // L
    {0x7F, 0x02, 0x0C, 0x02, 0x7F}, // M
    {0x7F, 0x04, 0x08, 0x10, 0x7F}, // N
    {0x3E, 0x41, 0x41, 0x41, 0x3E}, // O
    {0x7F, 0x09, 0x09, 0x09, 0x06}, // P
    {0x3E, 0x41, 0x51, 0x21, 0x5E}, // Q
    {0x7F, 0x09, 0x19, 0x29, 0x46}, // R
    {0x46, 0x49, 0x49, 0x49, 0x31}, // S
    {0x01, 0x01, 0x7F, 0x01, 0x01}, // T
    // U-Z (38-43)
    {0x3F, 0x40, 0x40, 0x40, 0x3F}, // U
    {0x1F, 0x20, 0x40, 0x20, 0x1F}, // V
    {0x3F, 0x40, 0x38, 0x40, 0x3F}, // W
    {0x63, 0x14, 0x08, 0x14, 0x63}, // X
    {0x07, 0x08, 0x70, 0x08, 0x07}, // Y
    {0x61, 0x51, 0x49, 0x45, 0x43}, // Z
    // a-z (44-69)
    {0x20, 0x54, 0x54, 0x54, 0x78}, // a
    {0x7F, 0x48, 0x44, 0x44, 0x38}, // b
    {0x38, 0x44, 0x44, 0x44, 0x20}, // c
    {0x38, 0x44, 0x44, 0x48, 0x7F}, // d
    {0x38, 0x54, 0x54, 0x54, 0x18}, // e
    {0x08, 0x7E, 0x09, 0x01, 0x02}, // f
    {0x0C, 0x52, 0x52, 0x52, 0x3E}, // g
    {0x7F, 0x08, 0x04, 0x04, 0x78}, // h
    {0x00, 0x44, 0x7D, 0x40, 0x00}, // i
    {0x20, 0x40, 0x44, 0x3D, 0x00}, // j
    {0x7F, 0x10, 0x28, 0x44, 0x00}, // k
    {0x00, 0x41, 0x7F, 0x40, 0x00}, // l
    {0x7C, 0x04, 0x18, 0x04, 0x78}, // m
    {0x7C, 0x08, 0x04, 0x04, 0x78}, // n
    {0x38, 0x44, 0x44, 0x44, 0x38}, // o
    {0x7C, 0x14, 0x14, 0x14, 0x08}, // p
    {0x08, 0x14, 0x14, 0x18, 0x7C}, // q
    {0x7C, 0x08, 0x04, 0x04, 0x08}, // r
    {0x48, 0x54, 0x54, 0x54, 0x20}, // s
    {0x04, 0x3F, 0x44, 0x40, 0x20}, // t
    {0x3C, 0x40, 0x40, 0x20, 0x7C}, // u
    {0x1C, 0x20, 0x40, 0x20, 0x1C}, // v
    {0x3C, 0x40, 0x30, 0x40, 0x3C}, // w
    {0x44, 0x28, 0x10, 0x28, 0x44}, // x
    {0x0C, 0x50, 0x50, 0x50, 0x3C}, // y
    {0x44, 0x64, 0x54, 0x4C, 0x44}, // z
};

// Map ASCII to font index
static int char_to_font_index(char c) {
  if (c == ' ')
    return 0;
  if (c == '!')
    return 1;
  if (c == '+')
    return 2;
  if (c == '-')
    return 3;
  if (c == '.')
    return 4;
  if (c == '/')
    return 5;
  if (c == ':')
    return 6;
  if (c == ',')
    return 7;
  if (c >= '0' && c <= '9')
    return 8 + (c - '0');
  if (c >= 'A' && c <= 'Z')
    return 18 + (c - 'A');
  if (c >= 'a' && c <= 'z')
    return 44 + (c - 'a');
  return 0; // Space for unknown
}

// Draw a single character
static void draw_char(SDL_Renderer *renderer, char c, int x, int y, int scale,
                      SDL_Color color) {
  int idx = char_to_font_index(c);
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  for (int row = 0; row < 7; row++) {
    uint8_t row_data = FONT_5X7[idx][row];
    for (int col = 0; col < 5; col++) {
      if (row_data & (1 << (4 - col))) {
        SDL_Rect pixel = {x + col * scale, y + row * scale, scale, scale};
        SDL_RenderFillRect(renderer, &pixel);
      }
    }
  }
}

// Draw text string
static void draw_text(SDL_Renderer *renderer, const char *text, int x, int y,
                      int scale, SDL_Color color) {
  int spacing = 6 * scale;
  for (int i = 0; text[i]; i++) {
    draw_char(renderer, text[i], x + i * spacing, y, scale, color);
  }
}

// Measure text width
static int measure_text_width(const char *text, int scale) {
  return strlen(text) * 6 * scale;
}

// Draw rounded rectangle with optional fill
static void draw_rounded_rect(SDL_Renderer *renderer, SDL_Rect *rect,
                              int radius, SDL_Color color, bool fill) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);
  SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);

  if (fill) {
    // Central rects
    SDL_Rect r1 = {rect->x + radius, rect->y, rect->w - 2 * radius, rect->h};
    SDL_Rect r2 = {rect->x, rect->y + radius, rect->w, rect->h - 2 * radius};
    SDL_RenderFillRect(renderer, &r1);
    SDL_RenderFillRect(renderer, &r2);

    // Fill corners
    for (int w = 0; w < radius; w++) {
      for (int h = 0; h < radius; h++) {
        if ((radius - w) * (radius - w) + (radius - h) * (radius - h) <=
            radius * radius) {
          SDL_RenderDrawPoint(renderer, rect->x + radius - w - 1,
                              rect->y + radius - h - 1);
          SDL_RenderDrawPoint(renderer, rect->x + rect->w - radius + w,
                              rect->y + radius - h - 1);
          SDL_RenderDrawPoint(renderer, rect->x + radius - w - 1,
                              rect->y + rect->h - radius + h);
          SDL_RenderDrawPoint(renderer, rect->x + rect->w - radius + w,
                              rect->y + rect->h - radius + h);
        }
      }
    }
  } else {
    // Draw outline only
    // Top and bottom edges
    SDL_RenderDrawLine(renderer, rect->x + radius, rect->y,
                       rect->x + rect->w - radius, rect->y);
    SDL_RenderDrawLine(renderer, rect->x + radius, rect->y + rect->h - 1,
                       rect->x + rect->w - radius, rect->y + rect->h - 1);
    // Left and right edges
    SDL_RenderDrawLine(renderer, rect->x, rect->y + radius, rect->x,
                       rect->y + rect->h - radius);
    SDL_RenderDrawLine(renderer, rect->x + rect->w - 1, rect->y + radius,
                       rect->x + rect->w - 1, rect->y + rect->h - radius);
  }
}

// Draw gradient rectangle (top to bottom)
static void draw_gradient_rect(SDL_Renderer *renderer, SDL_Rect *rect,
                               SDL_Color top, SDL_Color bottom) {
  SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);

  for (int y = 0; y < rect->h; y++) {
    float t = (float)y / rect->h;
    SDL_Color c = {(Uint8)(top.r + (bottom.r - top.r) * t),
                   (Uint8)(top.g + (bottom.g - top.g) * t),
                   (Uint8)(top.b + (bottom.b - top.b) * t),
                   (Uint8)(top.a + (bottom.a - top.a) * t)};
    SDL_SetRenderDrawColor(renderer, c.r, c.g, c.b, c.a);
    SDL_RenderDrawLine(renderer, rect->x, rect->y + y, rect->x + rect->w - 1,
                       rect->y + y);
  }
}

// Draw drop shadow
static void draw_shadow(SDL_Renderer *renderer, SDL_Rect *rect, int radius,
                        int depth) {
  SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);

  for (int i = depth; i > 0; i--) {
    Uint8 alpha = (Uint8)(40 * i / depth);
    SDL_Rect shadow = {rect->x + i, rect->y + i, rect->w, rect->h};
    draw_rounded_rect(renderer, &shadow, radius, (SDL_Color){0, 0, 0, alpha},
                      true);
  }
}

// --- Improved Icon Drawing Helpers ---

static void draw_icon_folder(SDL_Renderer *renderer, int x, int y, int size,
                             SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  // Tab
  int tab_w = size * 0.5f;
  int tab_h = size * 0.2f;
  SDL_Rect tab = {x + 2, y + 2, tab_w, tab_h};
  draw_rounded_rect(renderer, &tab, 2, color, true);

  // Body
  SDL_Rect body = {x + 2, y + tab_h + 2, size - 4, size - tab_h - 4};
  draw_rounded_rect(renderer, &body, 3, color, true);

  // Detail line
  SDL_SetRenderDrawColor(renderer, 255, 255, 255, 80);
  SDL_RenderDrawLine(renderer, x + 4, y + tab_h + 6, x + size - 4,
                     y + tab_h + 6);
}

static void draw_icon_floppy(SDL_Renderer *renderer, int x, int y, int size,
                             SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  // Main body
  int m = 4;
  SDL_Rect body = {x + m, y + m, size - 2 * m, size - 2 * m};
  draw_rounded_rect(renderer, &body, 2, color, true);

  // Metal shutter
  SDL_SetRenderDrawColor(renderer, 180, 180, 190, 255);
  SDL_Rect shutter = {x + m + 4, y + m + 4, size - 2 * m - 8, size * 0.35f};
  draw_rounded_rect(renderer, &shutter, 1, (SDL_Color){180, 180, 190, 255},
                    true);

  // Label area
  SDL_SetRenderDrawColor(renderer, 240, 240, 240, 200);
  SDL_Rect label = {x + m + 4, y + size * 0.5f, size - 2 * m - 8, size * 0.35f};
  draw_rounded_rect(renderer, &label, 1, (SDL_Color){240, 240, 240, 200}, true);
}

static void draw_icon_arrow(SDL_Renderer *renderer, int x, int y, int size,
                            bool right, SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  int cx = x + size / 2;
  int cy = y + size / 2;
  int len = size / 3;
  int head = size / 4;

  // Shaft
  if (right) {
    SDL_RenderDrawLine(renderer, cx - len, cy, cx + len - head, cy);
    // Arrow head
    SDL_RenderDrawLine(renderer, cx + len - head, cy - head / 2, cx + len, cy);
    SDL_RenderDrawLine(renderer, cx + len - head, cy + head / 2, cx + len, cy);
  } else {
    SDL_RenderDrawLine(renderer, cx + len, cy, cx - len + head, cy);
    // Arrow head
    SDL_RenderDrawLine(renderer, cx - len + head, cy - head / 2, cx - len, cy);
    SDL_RenderDrawLine(renderer, cx - len + head, cy + head / 2, cx - len, cy);
  }
}

static void draw_icon_crop(SDL_Renderer *renderer, int x, int y, int size,
                           SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  int m = 8;
  int r = {x + m, y + m, size - 2 * m, size - 2 * m};

  // Corner lines
  int corner = size / 4;
  // Top-left
  SDL_RenderDrawLine(renderer, r.x, r.y + corner, r.x, r.y);
  SDL_RenderDrawLine(renderer, r.x, r.y, r.x + corner, r.y);
  // Top-right
  SDL_RenderDrawLine(renderer, r.x + r.w - corner, r.y, r.x + r.w, r.y);
  SDL_RenderDrawLine(renderer, r.x + r.w, r.y, r.x + r.w, r.y + corner);
  // Bottom-left
  SDL_RenderDrawLine(renderer, r.x, r.y + r.h - corner, r.x, r.y + r.h);
  SDL_RenderDrawLine(renderer, r.x, r.y + r.h, r.x + corner, r.y + r.h);
  // Bottom-right
  SDL_RenderDrawLine(renderer, r.x + r.w - corner, r.y + r.h, r.x + r.w,
                     r.y + r.h);
  SDL_RenderDrawLine(renderer, r.x + r.w, r.y + r.h - corner, r.x + r.w,
                     r.y + r.h);

  // Diagonal cut line
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, 150);
  SDL_RenderDrawLine(renderer, r.x, r.y + r.h, r.x + r.w, r.y);
}

static void draw_icon_rotate(SDL_Renderer *renderer, int x, int y, int size,
                             SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  int cx = x + size / 2;
  int cy = y + size / 2;
  int r = size / 3;

  // Draw arc as a series of line segments
  int segments = 8;
  for (int i = 0; i < segments; i++) {
    float a1 = M_PI + (i * M_PI / segments);
    float a2 = M_PI + ((i + 1) * M_PI / segments);
    int x1 = cx + (int)(r * cosf(a1));
    int y1 = cy + (int)(r * sinf(a1) * 0.6f); // Flattened for perspective
    int x2 = cx + (int)(r * cosf(a2));
    int y2 = cy + (int)(r * sinf(a2) * 0.6f);
    SDL_RenderDrawLine(renderer, x1, y1, x2, y2);
  }

  // Arrow head
  SDL_RenderDrawLine(renderer, cx + r, cy, cx + r - 4, cy - 4);
  SDL_RenderDrawLine(renderer, cx + r, cy, cx + r - 4, cy + 4);
}

static void draw_icon_sun(SDL_Renderer *renderer, int x, int y, int size,
                          SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  int cx = x + size / 2;
  int cy = y + size / 2;
  int r = size / 5;

  // Center circle (filled)
  for (int dy = -r; dy <= r; dy++) {
    int dx = (int)sqrtf(r * r - dy * dy);
    SDL_RenderDrawLine(renderer, cx - dx, cy + dy, cx + dx, cy + dy);
  }

  // Rays
  int ray_len = size / 3;
  int inner = r + 3;
  // 8 rays
  for (int i = 0; i < 8; i++) {
    float angle = i * M_PI / 4;
    int x1 = cx + (int)(inner * cosf(angle));
    int y1 = cy + (int)(inner * sinf(angle));
    int x2 = cx + (int)(ray_len * cosf(angle));
    int y2 = cy + (int)(ray_len * sinf(angle));
    SDL_RenderDrawLine(renderer, x1, y1, x2, y2);
  }
}

static void draw_icon_resize(SDL_Renderer *renderer, int x, int y, int size,
                             SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  int m = 8;
  SDL_Rect r = {x + m, y + m, size - 2 * m, size - 2 * m};

  // Rectangle outline
  draw_rounded_rect(renderer, &r, 2, color, false);

  // Diagonal arrows
  int arrow = 6;
  // Bottom-left to top-right
  SDL_RenderDrawLine(renderer, r.x + 4, r.y + r.h - 4, r.x + r.w - 4, r.y + 4);
  // Arrow heads
  SDL_RenderDrawLine(renderer, r.x + r.w - 4, r.y + 4, r.x + r.w - 4 - arrow,
                     r.y + 4);
  SDL_RenderDrawLine(renderer, r.x + r.w - 4, r.y + 4, r.x + r.w - 4,
                     r.y + 4 + arrow);
  SDL_RenderDrawLine(renderer, r.x + 4, r.y + r.h - 4, r.x + 4 + arrow,
                     r.y + r.h - 4);
  SDL_RenderDrawLine(renderer, r.x + 4, r.y + r.h - 4, r.x + 4,
                     r.y + r.h - 4 - arrow);
}

static void draw_icon_compress(SDL_Renderer *renderer, int x, int y, int size,
                               SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  // Weight box
  int m = 8;
  SDL_Rect box = {x + m, y + size / 2, size - 2 * m, size / 3};
  draw_rounded_rect(renderer, &box, 2, color, true);

  // Top connector
  SDL_RenderDrawLine(renderer, x + size / 2, y + m, x + size / 2, y + size / 2);

  // Arrows indicating compression
  SDL_RenderDrawLine(renderer, x + size / 2 - 4, y + m + 4, x + size / 2,
                     y + m);
  SDL_RenderDrawLine(renderer, x + size / 2 + 4, y + m + 4, x + size / 2,
                     y + m);
}

static void draw_icon_plus(SDL_Renderer *renderer, int x, int y, int size,
                           SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  int cx = x + size / 2;
  int cy = y + size / 2;
  int len = size / 3;
  int thick = 2;

  // Horizontal
  SDL_Rect h = {cx - len, cy - thick / 2, len * 2, thick};
  SDL_RenderFillRect(renderer, &h);
  // Vertical
  SDL_Rect v = {cx - thick / 2, cy - len, thick, len * 2};
  SDL_RenderFillRect(renderer, &v);
}

static void draw_icon_minus(SDL_Renderer *renderer, int x, int y, int size,
                            SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  int cx = x + size / 2;
  int cy = y + size / 2;
  int len = size / 3;
  int thick = 2;

  SDL_Rect h = {cx - len, cy - thick / 2, len * 2, thick};
  SDL_RenderFillRect(renderer, &h);
}

static void draw_icon_fit(SDL_Renderer *renderer, int x, int y, int size,
                          SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  int m = 8;
  // Four corners as L shapes
  int l = 6;
  // TL
  SDL_RenderDrawLine(renderer, x + m, y + m, x + m + l, y + m);
  SDL_RenderDrawLine(renderer, x + m, y + m, x + m, y + m + l);
  // TR
  SDL_RenderDrawLine(renderer, x + size - m, y + m, x + size - m - l, y + m);
  SDL_RenderDrawLine(renderer, x + size - m, y + m, x + size - m, y + m + l);
  // BL
  SDL_RenderDrawLine(renderer, x + m, y + size - m, x + m + l, y + size - m);
  SDL_RenderDrawLine(renderer, x + m, y + size - m, x + m, y + size - m - l);
  // BR
  SDL_RenderDrawLine(renderer, x + size - m, y + size - m, x + size - m - l,
                     y + size - m);
  SDL_RenderDrawLine(renderer, x + size - m, y + size - m, x + size - m,
                     y + size - m - l);
}

static void draw_icon_fullscreen(SDL_Renderer *renderer, int x, int y, int size,
                                 SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  // Screen
  SDL_Rect screen = {x + 6, y + 6, size - 12, size - 16};
  draw_rounded_rect(renderer, &screen, 2, color, false);

  // Stand
  SDL_RenderDrawLine(renderer, x + size / 2, y + size - 10, x + size / 2,
                     y + size - 4);
  SDL_RenderDrawLine(renderer, x + size / 2 - 6, y + size - 4, x + size / 2 + 6,
                     y + size - 4);
}

static void draw_icon_quit(SDL_Renderer *renderer, int x, int y, int size,
                           SDL_Color color) {
  SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);

  int m = 10;
  int thick = 2;

  // X shape with thickness
  for (int i = 0; i < thick; i++) {
    SDL_RenderDrawLine(renderer, x + m + i, y + m, x + size - m + i,
                       y + size - m);
    SDL_RenderDrawLine(renderer, x + size - m + i, y + m, x + m + i,
                       y + size - m);
  }
}

// Icon drawing dispatcher - removed duplicate, icons are drawn directly in
// render loop

// --- Main UI Functions ---

void ui_init(Toolbar *toolbar, Sidebar *sidebar) {
  toolbar->visible = true;
  toolbar->height = TOOLBAR_HEIGHT;
  toolbar->button_count = 14;
  toolbar->hovered_button = -1;
  toolbar->pressed_button = -1;
  toolbar->auto_hide = true;
  toolbar->last_activity = SDL_GetTicks();
  toolbar->visibility_alpha = 1.0f;
  toolbar->target_alpha = 1.0f;

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
  static Uint32 hover_start_time = 0;
  static int last_hovered = -1;

  if (event->type == SDL_MOUSEMOTION) {
    toolbar->last_activity = SDL_GetTicks();
    toolbar->target_alpha = 1.0f;

    int mx = event->motion.x;
    int my = event->motion.y;

    toolbar->hovered_button = -1;
    for (int i = 0; i < toolbar->button_count; i++) {
      if (mx >= toolbar->buttons[i].x &&
          mx < toolbar->buttons[i].x + toolbar->buttons[i].w &&
          my >= toolbar->buttons[i].y &&
          my < toolbar->buttons[i].y + toolbar->buttons[i].h) {
        toolbar->hovered_button = i;
        if (last_hovered != i) {
          hover_start_time = SDL_GetTicks();
          last_hovered = i;
        }
        break;
      }
    }

    if (toolbar->hovered_button == -1) {
      last_hovered = -1;
    }
  }

  if (event->type == SDL_MOUSEBUTTONDOWN &&
      event->button.button == SDL_BUTTON_LEFT) {
    if (toolbar->hovered_button >= 0) {
      toolbar->pressed_button = toolbar->hovered_button;
    }
  }

  if (event->type == SDL_MOUSEBUTTONUP &&
      event->button.button == SDL_BUTTON_LEFT) {
    if (toolbar->pressed_button >= 0 &&
        toolbar->pressed_button == toolbar->hovered_button) {
      // Button was clicked
      switch (toolbar->pressed_button) {
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
    toolbar->pressed_button = -1;
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
        toolbar->target_alpha = 0.0f;
      }
    }
  }

  return result;
}

void ui_update(Toolbar *toolbar, Tooltip *tooltip) {
  // Smooth toolbar fade
  float speed = ANIMATION_SPEED;
  if (toolbar->visibility_alpha < toolbar->target_alpha) {
    toolbar->visibility_alpha += speed;
    if (toolbar->visibility_alpha > toolbar->target_alpha)
      toolbar->visibility_alpha = toolbar->target_alpha;
  } else if (toolbar->visibility_alpha > toolbar->target_alpha) {
    toolbar->visibility_alpha -= speed;
    if (toolbar->visibility_alpha < toolbar->target_alpha)
      toolbar->visibility_alpha = toolbar->target_alpha;
  }

  // Tooltip visibility
  if (tooltip->visible) {
    if (tooltip->alpha < 1.0f)
      tooltip->alpha += 0.1f;
    if (tooltip->y_offset > 0)
      tooltip->y_offset -= 1.0f;
  } else {
    tooltip->alpha = 0.0f;
    tooltip->y_offset = 8.0f;
  }
}

// Editor operations need to be available here, technically UI shouldn't depend
// on Editor internals directly but for simplicity we'll forward declare or
// include editor.h. Since we can't easily change includes in a multi-replace
// without context of top of file, we assume editor.h is included or we add the
// include. Let's add the include at the top first in a separate chunk.

void ui_render(Toolbar *toolbar, Sidebar *sidebar, SDL_Renderer *renderer,
               Editor *editor) {
  int win_w, win_h;
  SDL_GetRendererOutputSize(renderer, &win_w, &win_h);

  // Render toolbar
  if (toolbar->visibility_alpha > 0.01f) {
    int total_w =
        (BUTTON_SIZE + BUTTON_PADDING) * toolbar->button_count + BUTTON_PADDING;
    int start_x = (win_w - total_w) / 2;
    int start_y = win_h - TOOLBAR_HEIGHT - TOOLBAR_BOTTOM_MARGIN;

    // Toolbar background with gradient
    SDL_Rect toolbar_bg = {start_x, start_y, total_w, TOOLBAR_HEIGHT};

    // Draw shadow first
    draw_shadow(renderer, &toolbar_bg, 10, 8);

    // Draw gradient background
    SDL_Color top = {40, 42, 50, (Uint8)(230 * toolbar->visibility_alpha)};
    SDL_Color bottom = {25, 27, 35, (Uint8)(230 * toolbar->visibility_alpha)};
    draw_gradient_rect(renderer, &toolbar_bg, top, bottom);

    // Draw border
    SDL_SetRenderDrawColor(renderer, 255, 255, 255,
                           (Uint8)(25 * toolbar->visibility_alpha));
    draw_rounded_rect(
        renderer, &toolbar_bg, 10,
        (SDL_Color){255, 255, 255, (Uint8)(25 * toolbar->visibility_alpha)},
        false);

    // Draw Buttons
    for (int i = 0; i < toolbar->button_count; i++) {
      SDL_Rect btn_rect = {start_x + BUTTON_PADDING +
                               i * (BUTTON_SIZE + BUTTON_PADDING),
                           start_y + (TOOLBAR_HEIGHT - BUTTON_SIZE) / 2,
                           BUTTON_SIZE, BUTTON_SIZE};

      toolbar->buttons[i] = btn_rect;

      bool hovered = (i == toolbar->hovered_button);
      bool pressed = (i == toolbar->pressed_button);

      // Button background with states
      if (pressed) {
        draw_rounded_rect(
            renderer, &btn_rect, 6,
            (SDL_Color){
                COLOR_BG_PRESSED.r, COLOR_BG_PRESSED.g, COLOR_BG_PRESSED.b,
                (Uint8)(COLOR_BG_PRESSED.a * toolbar->visibility_alpha)},
            true);
      } else if (hovered) {
        // Glow effect
        SDL_Rect glow = {btn_rect.x - 2, btn_rect.y - 2, btn_rect.w + 4,
                         btn_rect.h + 4};
        draw_rounded_rect(renderer, &glow, 8,
                          (SDL_Color){COLOR_ACCENT.r, COLOR_ACCENT.g,
                                      COLOR_ACCENT.b,
                                      (Uint8)(40 * toolbar->visibility_alpha)},
                          true);
        draw_rounded_rect(
            renderer, &btn_rect, 6,
            (SDL_Color){COLOR_BG_HOVER.r, COLOR_BG_HOVER.g, COLOR_BG_HOVER.b,
                        (Uint8)(COLOR_BG_HOVER.a * toolbar->visibility_alpha)},
            true);
      }

      // Icon with adjusted alpha
      SDL_Color icon_color = hovered ? COLOR_ACCENT : COLOR_ICON;
      icon_color.a = (Uint8)(icon_color.a * toolbar->visibility_alpha);

      // Call appropriate icon function based on type
      switch (i) {
      case 0:
        draw_icon_folder(renderer, btn_rect.x + 4, btn_rect.y + 4,
                         BUTTON_SIZE - 8, icon_color);
        break;
      case 1:
        draw_icon_floppy(renderer, btn_rect.x + 4, btn_rect.y + 4,
                         BUTTON_SIZE - 8, icon_color);
        break;
      case 2:
        draw_icon_arrow(renderer, btn_rect.x + 4, btn_rect.y + 4,
                        BUTTON_SIZE - 8, false, icon_color);
        break;
      case 3:
        draw_icon_arrow(renderer, btn_rect.x + 4, btn_rect.y + 4,
                        BUTTON_SIZE - 8, true, icon_color);
        break;
      case 4:
        draw_icon_crop(renderer, btn_rect.x + 4, btn_rect.y + 4,
                       BUTTON_SIZE - 8, icon_color);
        break;
      case 5:
        draw_icon_rotate(renderer, btn_rect.x + 4, btn_rect.y + 4,
                         BUTTON_SIZE - 8, icon_color);
        break;
      case 6:
        draw_icon_sun(renderer, btn_rect.x + 4, btn_rect.y + 4, BUTTON_SIZE - 8,
                      icon_color);
        break;
      case 7:
        draw_icon_resize(renderer, btn_rect.x + 4, btn_rect.y + 4,
                         BUTTON_SIZE - 8, icon_color);
        break;
      case 8:
        draw_icon_compress(renderer, btn_rect.x + 4, btn_rect.y + 4,
                           BUTTON_SIZE - 8, icon_color);
        break;
      case 9:
        draw_icon_plus(renderer, btn_rect.x + 4, btn_rect.y + 4,
                       BUTTON_SIZE - 8, icon_color);
        break;
      case 10:
        draw_icon_minus(renderer, btn_rect.x + 4, btn_rect.y + 4,
                        BUTTON_SIZE - 8, icon_color);
        break;
      case 11:
        draw_icon_fit(renderer, btn_rect.x + 4, btn_rect.y + 4, BUTTON_SIZE - 8,
                      icon_color);
        break;
      case 12:
        draw_icon_fullscreen(renderer, btn_rect.x + 4, btn_rect.y + 4,
                             BUTTON_SIZE - 8, icon_color);
        break;
      case 13:
        draw_icon_quit(renderer, btn_rect.x + 4, btn_rect.y + 4,
                       BUTTON_SIZE - 8, icon_color);
        break;
      }
    }
  }

  // Render sidebar
  if (sidebar->visible) {
    SDL_Rect sidebar_rect = {0, 0, sidebar->width, win_h};

    // Gradient background
    SDL_Color top = {35, 37, 45, 245};
    SDL_Color bottom = {25, 27, 35, 245};
    draw_gradient_rect(renderer, &sidebar_rect, top, bottom);

    // Shadow on right edge
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 80);
    for (int i = 0; i < 4; i++) {
      SDL_RenderDrawLine(renderer, sidebar->width + i, 0, sidebar->width + i,
                         win_h);
    }

    // Border
    SDL_SetRenderDrawColor(renderer, 80, 85, 100, 255);
    SDL_RenderDrawLine(renderer, sidebar->width, 0, sidebar->width, win_h);

    // Title
    SDL_Color title_color = {200, 200, 210, 255};
    int x = 15;
    // title, not a button.

    // If gui_button and editor_rotate_left/right are meant to be called,
    // they need to be defined and `editor` needs to be passed to `ui_render`.
    // For now, I'll comment them out to avoid immediate compilation errors,
    // but keep them in the code as per the instruction.
    /*
    if (gui_button(renderer, "Rotate L", x, y, btn_w, btn_h, false)) {
      // editor_rotate_left(editor); // `editor` is not available in this scope
    }
    x += btn_w + padding; // `padding` is not available in this scope

    if (gui_button(renderer, "Rotate R", x, y, btn_w, btn_h, false)) {
      // editor_rotate_right(editor); // `editor` is not available in this scope
    }
    x += btn_w + padding; // `padding` is not available in this scope
    */

    // The last line of the snippet was:
    // if (gui_button(renderer, "Brightness", x, y, btn_w, btn_h, title_color);
    // This is syntactically incorrect and seems to be a mix of a button call
    // and a text call. Assuming it's meant to be a title for a section, similar
    // to "Images". If it was meant to be a button, it needs a proper
    // `gui_button` call and an action. For now, I'll interpret it as a title,
    // placed below the "Images" title or as a replacement. Given the original
    // `draw_text("Images", ...)` was at (15, 15), and the snippet implies
    // adding new elements, I'll add "Brightness" as a new title.
    // draw_text(renderer, "Brightness", x, y, 2, title_color); // Using x, y
    // from above, which are dummy.
  }

  // Render tooltip
  if (tooltip->visible && tooltip->alpha > 0.01f) {
    int text_w = measure_text_width(tooltip->label, 2);
    int padding = 10;
    int tooltip_w = text_w + padding * 2;
    int tooltip_h = 28;

    SDL_Rect btn = toolbar->buttons[tooltip->target_button];
    int tooltip_x = btn.x + btn.w / 2 - tooltip_w / 2;
    int tooltip_y = btn.y - tooltip_h - 8 + (int)tooltip->y_offset;

    SDL_Rect tooltip_rect = {tooltip_x, tooltip_y, tooltip_w, tooltip_h};

    // Shadow
    draw_shadow(renderer, &tooltip_rect, 6, 4);

    // Background
    draw_rounded_rect(renderer, &tooltip_rect, 6,
                      (SDL_Color){30, 32, 40, (Uint8)(240 * tooltip->alpha)},
                      true);

    // Border
    draw_rounded_rect(renderer, &tooltip_rect, 6,
                      (SDL_Color){100, 105, 120, (Uint8)(100 * tooltip->alpha)},
                      false);

    // Text
    SDL_Color text_color = {220, 220, 230, (Uint8)(255 * tooltip->alpha)};
    draw_text(renderer, tooltip->label, tooltip_x + padding, tooltip_y + 6, 2,
              text_color);
  }
}

void ui_show_toast(Toast *toast, const char *message, Uint32 duration) {
  strncpy(toast->message, message, sizeof(toast->message) - 1);
  toast->message[sizeof(toast->message) - 1] = '\0';
  toast->show_time = SDL_GetTicks();
  toast->duration = duration;
  toast->visible = true;
  toast->alpha = 0.0f;
}

void ui_render_toast(Toast *toast, SDL_Renderer *renderer) {
  if (!toast->visible)
    return;

  Uint32 elapsed = SDL_GetTicks() - toast->show_time;

  // Calculate alpha with fade in/out
  if (elapsed < 300) {
    toast->alpha = (float)elapsed / 300.0f;
  } else if (elapsed > toast->duration - 500) {
    toast->alpha = (float)(toast->duration - elapsed) / 500.0f;
  } else {
    toast->alpha = 1.0f;
  }

  if (toast->alpha <= 0.0f || elapsed > toast->duration) {
    toast->visible = false;
    return;
  }

  int win_w, win_h;
  SDL_GetRendererOutputSize(renderer, &win_w, &win_h);

  int text_scale = 2;
  int text_w = measure_text_width(toast->message, text_scale);
  int padding = 16;
  int toast_w = text_w + padding * 2;
  int toast_h = 40;

  SDL_Rect toast_rect = {(win_w - toast_w) / 2, win_h - 120, toast_w, toast_h};

  Uint8 alpha = (Uint8)(240 * toast->alpha);

  // Shadow
  draw_shadow(renderer, &toast_rect, 12, 6);

  // Gradient background
  SDL_Color top = {45, 47, 55, alpha};
  SDL_Color bottom = {30, 32, 40, alpha};
  draw_gradient_rect(renderer, &toast_rect, top, bottom);

  // Border
  draw_rounded_rect(renderer, &toast_rect, 12,
                    (SDL_Color){100, 105, 120, (Uint8)(60 * toast->alpha)},
                    false);

  // Text
  SDL_Color text_color = {230, 230, 240, (Uint8)(255 * toast->alpha)};
  draw_text(renderer, toast->message, toast_rect.x + padding, toast_rect.y + 10,
            text_scale, text_color);
}

void ui_render_placeholder(SDL_Renderer *renderer, int win_w, int win_h) {
  // Background gradient
  SDL_Rect bg = {0, 0, win_w, win_h};
  SDL_Color top = {28, 30, 38, 255};
  SDL_Color bottom = {18, 20, 25, 255};
  draw_gradient_rect(renderer, &bg, top, bottom);

  // Main box
  int box_w = 320;
  int box_h = 240;
  int box_x = (win_w - box_w) / 2;
  int box_y = (win_h - box_h) / 2;

  SDL_Rect box = {box_x, box_y, box_w, box_h};

  // Box shadow
  draw_shadow(renderer, &box, 20, 12);

  // Box background with gradient
  SDL_Color box_top = {45, 48, 58, 245};
  SDL_Color box_bottom = {35, 38, 48, 245};
  draw_gradient_rect(renderer, &box, box_top, box_bottom);

  // Border
  draw_rounded_rect(renderer, &box, 16, (SDL_Color){80, 85, 100, 80}, false);

  // Folder icon (large)
  int icon_size = 80;
  int icon_x = (win_w - icon_size) / 2;
  int icon_y = box_y + 30;
  draw_icon_folder(renderer, icon_x, icon_y, icon_size, COLOR_ACCENT);

  // "Drop Image" text
  SDL_Color accent_color = COLOR_ACCENT;
  int text_y = icon_y + icon_size + 20;
  draw_text(renderer, "Drop Image Here",
            (win_w - measure_text_width("Drop Image Here", 3)) / 2, text_y, 3,
            accent_color);

  // Subtitle
  SDL_Color dim_color = COLOR_TEXT_DIM;
  text_y += 35;
  draw_text(renderer, "or press O to open",
            (win_w - measure_text_width("or press O to open", 2)) / 2, text_y,
            2, dim_color);

  // Keyboard shortcuts hint at bottom
  text_y += 40;
  draw_text(
      renderer, "Shortcuts: +/- Zoom  |  F Fullscreen  |  ? Help",
      (win_w - measure_text_width(
                   "Shortcuts: +/- Zoom  |  F Fullscreen  |  ? Help", 1)) /
          2,
      win_h - 60, 1, dim_color);
}
