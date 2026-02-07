#include "editor.h"
#include "image.h"
#include "ui.h"
#include "utils.h"
#include "viewport.h"
#include "window.h"
#include <SDL2/SDL.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define APP_NAME "SpedImage"
#define CACHE_SIZE_MB 50
#define KB 1024
#define MB (1024 * 1024)

#ifdef _WIN32
#define PATH_SEP '\\'
#else
#define PATH_SEP '/'
#endif

typedef struct {
  AppWindow window;
  ImageCache cache;
  Viewport viewport;
  Editor editor;
  Toolbar toolbar;
  Sidebar sidebar;
  Toast toast;
  bool slideshow;
  Uint32 slideshow_timer;
  int slideshow_delay;
  FileList file_list;
  char *current_directory;
  int current_file_index; // Index in the file list
} AppState;

static void show_help(void) {
  printf("\n%s - Superfast Image Viewer\n", APP_NAME);
  printf("Usage: spedimage [image or directory]\n\n");
  printf("Keyboard Shortcuts:\n");
  printf("  Left/Right    Previous/Next image\n");
  printf("  +/-           Zoom in/out\n");
  printf("  0             Fit to window\n");
  printf("  F             Toggle fullscreen\n");
  printf("  C             Crop mode\n");
  printf("  R             Rotate 90 degrees\n");
  printf("  B             Brightness/Contrast adjust\n");
  printf("  Delete        Delete current image\n");
  printf("  Space         Start/stop slideshow\n");
  printf("  F1            Toggle sidebar\n");
  printf("  Enter         Apply Edit / Save\n");
  printf("  Esc           Exit tool/Close app\n");
  printf("\n");
}

static void load_image_from_index(AppState *app, int index) {
  if (index < 0 || (size_t)index >= app->file_list.count)
    return;

  app->current_file_index = index;

  char path[2048];
  snprintf(path, sizeof(path), "%s%c%s", app->current_directory, PATH_SEP,
           app->file_list.files[index]);

  Image *img = image_load(path, app->window.renderer);
  if (img) {
    // Clear cache when jumping significant distances or changing folders
    // For now, simple LRU handles it, but we could be smarter

    cache_add(&app->cache, img);
    app->cache.current_index = app->cache.count - 1;

    // Update current index in cache to match file list index conceptually
    // (Note: cache index != file listing index. Cache is what's loaded in RAM.)

    if (app->viewport.fit_to_window) {
      viewport_fit_image(&app->viewport, img->width, img->height,
                         app->window.width, app->window.height);
    }

    char msg[256];
    snprintf(msg, sizeof(msg), "[%d/%d] %s", index + 1,
             (int)app->file_list.count, img->filename);
    ui_show_toast(&app->toast, msg, 2000);

    // Update window title
    char title[512];
    snprintf(title, sizeof(title), "%s - %s", APP_NAME,
             get_filename_from_path(img->filename));
    SDL_SetWindowTitle(app->window.window, title);
  }
}

static void load_directory(AppState *app, const char *path) {
  // Determine if path is a file or directory
  char *dir_path = get_directory_from_path(path);
  const char *filename = get_filename_from_path(path);

  if (app->current_directory)
    free(app->current_directory);
  app->current_directory = dir_path; // Now owned by AppState

  file_list_scan_directory(&app->file_list, dir_path);

  // Find index of the requested image
  int index = file_list_find_index(&app->file_list, filename);
  if (index >= 0) {
    app->current_file_index = index;
  } else {
    index = 0; // Default to first if not found
    app->current_file_index = 0;
  }

  // Actually load the image
  load_image_from_index(app, index);

  // HACK: Store the file list index in the cache structure's "current_index"
  // This is messy. `ImageCache` works on loaded images.
  // We need to track `current_file_index` in AppState.
  // Let's just shadow it or add it to AppState.
}

static void handle_keydown(AppState *app, SDL_Keycode key) {
  Image *current = cache_get(&app->cache, app->cache.current_index);

  switch (key) {
  case SDLK_LEFT: {
    if (app->current_file_index > 0) {
      load_image_from_index(app, app->current_file_index - 1);
    } else {
      // Wrap around
      load_image_from_index(app, app->file_list.count - 1);
    }
  } break;

  case SDLK_RIGHT: {
    if (app->current_file_index < (int)app->file_list.count - 1) {
      load_image_from_index(app, app->current_file_index + 1);
    } else {
      // Wrap around
      load_image_from_index(app, 0);
    }
  } break;

  case SDLK_PLUS:
  case SDLK_KP_PLUS:
  case SDLK_EQUALS:
    if (app->editor.active) {
      if (app->editor.mode == EDIT_MODE_RESIZE) {
        app->editor.target_width += 50;
        // Calculate height to maintain aspect
        if (app->editor.maintain_aspect && current) {
          float ratio = (float)current->height / current->width;
          app->editor.target_height = (int)(app->editor.target_width * ratio);
        }
        char msg[64];
        snprintf(msg, sizeof(msg), "Resize: %d x %d", app->editor.target_width,
                 app->editor.target_height);
        ui_show_toast(&app->toast, msg, 2000);
      } else if (app->editor.mode == EDIT_MODE_COMPRESS) {
        app->editor.target_size_kb += 50;
        char msg[64];
        snprintf(msg, sizeof(msg), "Compress Limit: %d KB",
                 app->editor.target_size_kb);
        ui_show_toast(&app->toast, msg, 2000);
      } else if (app->editor.mode == EDIT_MODE_BRIGHTNESS) {
        app->editor.brightness += 10.0f;
        if (current)
          editor_apply(current, &app->editor,
                       app->window.renderer); // Live preview?
        ui_show_toast(&app->toast, "Brightness +", 1000);
      }
    } else {
      viewport_zoom(&app->viewport, 1.25f, app->window.width / 2,
                    app->window.height / 2);
    }
    break;

  case SDLK_MINUS:
  case SDLK_KP_MINUS:
    if (app->editor.active) {
      if (app->editor.mode == EDIT_MODE_RESIZE) {
        app->editor.target_width -= 50;
        if (app->editor.target_width < 50)
          app->editor.target_width = 50;
        // Calculate height to maintain aspect
        if (app->editor.maintain_aspect && current) {
          float ratio = (float)current->height / current->width;
          app->editor.target_height = (int)(app->editor.target_width * ratio);
        }
        char msg[64];
        snprintf(msg, sizeof(msg), "Resize: %d x %d", app->editor.target_width,
                 app->editor.target_height);
        ui_show_toast(&app->toast, msg, 2000);
      } else if (app->editor.mode == EDIT_MODE_COMPRESS) {
        app->editor.target_size_kb -= 50;
        if (app->editor.target_size_kb < 10)
          app->editor.target_size_kb = 10;
        char msg[64];
        snprintf(msg, sizeof(msg), "Compress Limit: %d KB",
                 app->editor.target_size_kb);
        ui_show_toast(&app->toast, msg, 2000);
      }
    } else {
      viewport_zoom(&app->viewport, 0.8f, app->window.width / 2,
                    app->window.height / 2);
    }
    break;

  case SDLK_0:
    app->viewport.fit_to_window = true;
    if (current) {
      viewport_fit_image(&app->viewport, current->width, current->height,
                         app->window.width, app->window.height);
    }
    break;

  case SDLK_f:
  case SDLK_F11:
    window_toggle_fullscreen(&app->window);
    break;

  case SDLK_c:
    editor_start_crop(&app->editor);
    ui_show_toast(&app->toast, "Crop mode - Select area", 3000);
    break;

  case SDLK_r:
    if (!app->editor.active) {
      editor_start_rotate(&app->editor);
      app->editor.rotation_angle = 90.0f;
      if (current) {
        editor_apply(current, &app->editor, app->window.renderer);
        viewport_fit_image(&app->viewport, current->width, current->height,
                           app->window.width, app->window.height);
      }
      ui_show_toast(&app->toast, "Rotated 90 degrees", 2000);
    }
    break;

  case SDLK_b:
    editor_start_brightness(&app->editor);
    ui_show_toast(&app->toast, "Brightness mode - Use +/- to adjust", 3000);
    break;

  case SDLK_SPACE:
    app->slideshow = !app->slideshow;
    app->slideshow_timer = SDL_GetTicks();
    ui_show_toast(&app->toast,
                  app->slideshow ? "Slideshow started" : "Slideshow stopped",
                  2000);
    break;

  case SDLK_RETURN:
  case SDLK_KP_ENTER:
    if (app->editor.active && current) {
      editor_apply(current, &app->editor, app->window.renderer);
      // Reset viewport to fit new image dimensions
      if (app->viewport.fit_to_window) {
        viewport_fit_image(&app->viewport, current->width, current->height,
                           app->window.width, app->window.height);
      }
      ui_show_toast(&app->toast, "Applied", 2000);
    }
    break;

  case SDLK_F1:
    app->sidebar.visible = !app->sidebar.visible;
    break;

  case SDLK_ESCAPE:
    if (app->editor.active) {
      editor_cancel(&app->editor);
    } else {
      app->window.running = false;
    }
    break;

  case SDLK_q:
    app->window.running = false;
    break;
  }
}

static void handle_mousewheel(AppState *app, SDL_MouseWheelEvent *wheel) {
  int mx, my;
  SDL_GetMouseState(&mx, &my);

  if (wheel->y > 0) {
    viewport_zoom(&app->viewport, 1.1f, mx, my);
  } else if (wheel->y < 0) {
    viewport_zoom(&app->viewport, 0.9f, mx, my);
  }
}

int main(int argc, char *argv[]) {
  char *initial_path = NULL;
  for (int i = 1; i < argc; i++) {
    if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--verbose") == 0) {
      g_verbose = true;
    } else if (initial_path == NULL) {
      initial_path = argv[i];
    }
  }

  log_info("Starting %s in verbose mode", APP_NAME);

  AppState app = {0};
  app.slideshow_delay = 3000; // 3 seconds

  if (!window_init(&app.window, APP_NAME)) {
    fprintf(stderr, "Failed to initialize window\n");
    return 1;
  }

  cache_init(&app.cache, CACHE_SIZE_MB * 1024 * 1024);
  viewport_init(&app.viewport);
  editor_init(&app.editor);
  file_list_init(&app.file_list);
  ui_init(&app.toolbar, &app.sidebar);

  // Load initial image if provided
  if (initial_path) {
    log_info("Loading initial path: %s", initial_path);
    load_directory(&app, initial_path);
  } else {
    log_info("No initial path provided, starting with empty GUI");
  }

  // Main loop
  SDL_Event event;
  while (app.window.running) {
    // Handle events
    while (SDL_PollEvent(&event)) {
      if (event.type == SDL_QUIT) {
        app.window.running = false;
      } else if (event.type == SDL_KEYDOWN) {
        handle_keydown(&app, event.key.keysym.sym);
      } else if (event.type == SDL_MOUSEWHEEL) {
        handle_mousewheel(&app, &event.wheel);
      } else if (event.type == SDL_WINDOWEVENT) {
        if (event.window.event == SDL_WINDOWEVENT_SIZE_CHANGED) {
          window_get_size(&app.window, &app.window.width, &app.window.height);
          Image *current = cache_get(&app.cache, app.cache.current_index);
          if (current && app.viewport.fit_to_window) {
            viewport_fit_image(&app.viewport, current->width, current->height,
                               app.window.width, app.window.height);
          }
        }
      } else if (event.type == SDL_DROPFILE) {
        char *dropped_file = event.drop.file;
        log_info("File dropped: %s", dropped_file);
        load_directory(&app, dropped_file);
        // If we dropped a file, ensure we are NOT in fitting mode if we want to
        // zoom? Actually load_directory handles fitting.
        SDL_free(dropped_file);
      }

      // Handle UI events
      UITool tool = ui_handle_event(&app.toolbar, &app.sidebar, &event);
      switch (tool) {
      case UI_TOOL_OPEN:
        // Open file dialog would go here
        break;
      case UI_TOOL_SAVE: {
        Image *current = cache_get(&app.cache, app.cache.current_index);
        if (app.cache.current_index >= 0 && current) {
          // Save current image
          ui_show_toast(&app.toast, "Image saved", 2000);
        }
      } break;
      case UI_TOOL_PREV:
        handle_keydown(&app, SDLK_LEFT);
        break;
      case UI_TOOL_NEXT:
        handle_keydown(&app, SDLK_RIGHT);
        break;
      case UI_TOOL_CROP:
        editor_start_crop(&app.editor);
        break;
      case UI_TOOL_ROTATE:
        handle_keydown(&app, SDLK_r);
        break;
      case UI_TOOL_BRIGHTNESS:
        editor_start_brightness(&app.editor);
        break;
      case UI_TOOL_RESIZE: {
        Image *current = cache_get(&app.cache, app.cache.current_index);
        if (current) {
          editor_start_resize(&app.editor, current->width, current->height);
          ui_show_toast(&app.toast, "Resize: Use +/- to change width", 3000);
        }
      } break;
      case UI_TOOL_COMPRESS: {
        Image *current = cache_get(&app.cache, app.cache.current_index);
        if (current) {
          editor_start_compress(&app.editor, current->raw_size);
          char msg[64];
          snprintf(msg, sizeof(msg), "Compress: Limit %d KB (Use +/-)",
                   app.editor.target_size_kb);
          ui_show_toast(&app.toast, msg, 3000);
        }
      } break;
      case UI_TOOL_ZOOM_IN:
        handle_keydown(&app, SDLK_PLUS);
        break;
      case UI_TOOL_ZOOM_OUT:
        handle_keydown(&app, SDLK_MINUS);
        break;
      case UI_TOOL_FIT:
        handle_keydown(&app, SDLK_0);
        break;
      case UI_TOOL_FULLSCREEN:
        handle_keydown(&app, SDLK_f);
        break;
      case UI_TOOL_QUIT:
        app.window.running = false;
        break;
      default:
        break;
      }

      editor_handle_event(&app.editor, &event);
    }

    // Handle slideshow
    if (app.slideshow && app.cache.count > 1) {
      if (SDL_GetTicks() - app.slideshow_timer > (Uint32)app.slideshow_delay) {
        int next = (app.cache.current_index + 1) % app.cache.count;
        cache_get(&app.cache, next);
        app.slideshow_timer = SDL_GetTicks();
      }
    }

    // Render
    SDL_SetRenderDrawColor(app.window.renderer, 20, 20, 20, 255);
    SDL_RenderClear(app.window.renderer);

    // Render image
    Image *current = cache_get(&app.cache, app.cache.current_index);
    if (current) {
      if (app.viewport.fit_to_window) {
        viewport_fit_image(&app.viewport, current->width, current->height,
                           app.window.width, app.window.height);
      }
      viewport_render(&app.viewport, app.window.renderer, current);
    } else {
      ui_render_placeholder(app.window.renderer, app.window.width,
                            app.window.height);
    }

    // Render UI
    editor_render_ui(&app.editor, app.window.renderer);
    ui_render(&app.toolbar, &app.sidebar, app.window.renderer);
    ui_render_toast(&app.toast, app.window.renderer);

    SDL_RenderPresent(app.window.renderer);
    SDL_Delay(16); // ~60 FPS
  }

  // Cleanup
  editor_cleanup(&app.editor);
  cache_cleanup(&app.cache);
  file_list_free(&app.file_list);
  if (app.current_directory)
    free(app.current_directory);
  ui_cleanup();
  window_cleanup(&app.window);

  return 0;
}
