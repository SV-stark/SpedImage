#include "utils.h"
#include <ctype.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>


bool g_verbose = false;

void log_info(const char *fmt, ...) {
  if (!g_verbose)
    return;
  va_list args;
  va_start(args, fmt);
  vprintf(fmt, args);
  va_end(args);
  printf("\n");
}

#ifdef _WIN32
#include <windows.h>
#define PATH_SEP '\\'
#define strcasecmp _stricmp
#else
#include <dirent.h>
#include <sys/stat.h>
#include <unistd.h>
#define PATH_SEP '/'
#endif

void file_list_init(FileList *list) {
  list->files = NULL;
  list->count = 0;
  list->capacity = 0;
}

void file_list_free(FileList *list) {
  if (list->files) {
    for (size_t i = 0; i < list->count; i++) {
      free(list->files[i]);
    }
    free(list->files);
  }
  list->files = NULL;
  list->count = 0;
  list->capacity = 0;
}

static void add_file(FileList *list, const char *filename) {
  if (list->count >= list->capacity) {
    size_t new_capacity = list->capacity == 0 ? 16 : list->capacity * 2;
    char **new_files = realloc(list->files, new_capacity * sizeof(char *));
    if (!new_files)
      return;
    list->files = new_files;
    list->capacity = new_capacity;
  }
  list->files[list->count++] = strdup(filename);
}

// Simple case-insensitive comparison
static int compare_files(const void *a, const void *b) {
  const char *sa = *(const char **)a;
  const char *sb = *(const char **)b;

  while (*sa && *sb) {
    if (tolower(*sa) != tolower(*sb)) {
      return tolower(*sa) - tolower(*sb);
    }
    sa++;
    sb++;
  }
  return tolower(*sa) - tolower(*sb);
}

static bool is_image_ext(const char *filename) {
  const char *dot = strrchr(filename, '.');
  if (!dot)
    return false;
  const char *ext = dot + 1;

  const char *extensions[] = {"jpg",  "jpeg", "png",  "bmp", "gif", "tga",
                              "hdr",  "psd",  "pic",  "pnm", "svg", "heic",
                              "avif", "webp", "tiff", "tif", "raw", "cr2",
                              "nef",  "arw",  "dng",  NULL};

  for (int i = 0; extensions[i]; i++) {
    if (strcasecmp(ext, extensions[i]) == 0)
      return true;
  }
  return false;
}

bool file_list_scan_directory(FileList *list, const char *directory_path) {
  log_info("Scanning directory: %s", directory_path);
  file_list_free(list);

#ifdef _WIN32
  char search_path[2048];
  snprintf(search_path, sizeof(search_path), "%s\\*", directory_path);

  WIN32_FIND_DATAW find_data;
  wchar_t wpath[2048];
  MultiByteToWideChar(CP_UTF8, 0, search_path, -1, wpath, 2048);

  HANDLE hFind = FindFirstFileW(wpath, &find_data);
  if (hFind == INVALID_HANDLE_VALUE)
    return false;

  do {
    if (find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
      continue;

    char filename[1024];
    WideCharToMultiByte(CP_UTF8, 0, find_data.cFileName, -1, filename,
                        sizeof(filename), NULL, NULL);

    if (is_image_ext(filename)) {
      add_file(list, filename);
    }
  } while (FindNextFileW(hFind, &find_data));

  FindClose(hFind);
#else
  DIR *dir = opendir(directory_path);
  if (!dir)
    return false;

  struct dirent *ent;
  while ((ent = readdir(dir)) != NULL) {
    if (ent->d_type == DT_DIR)
      continue;

    if (is_image_ext(ent->d_name)) {
      add_file(list, ent->d_name);
    }
  }
  closedir(dir);
#endif

  // Sort files
  log_info("Found %d images", (int)list->count);
  qsort(list->files, list->count, sizeof(char *), compare_files);
  return true;
}

int file_list_find_index(FileList *list, const char *filename) {
  for (size_t i = 0; i < list->count; i++) {
    if (strcasecmp(list->files[i], filename) == 0) {
      return (int)i;
    }
  }
  return -1;
}

char *get_directory_from_path(const char *filepath) {
  const char *last_sep = strrchr(filepath, PATH_SEP);
  if (!last_sep)
    return strdup(".");

  size_t len = last_sep - filepath;
  char *dir = malloc(len + 1);
  if (dir) {
    strncpy(dir, filepath, len);
    dir[len] = '\0';
  }
  return dir;
}

const char *get_filename_from_path(const char *filepath) {
  const char *last_sep = strrchr(filepath, PATH_SEP);
  return last_sep ? last_sep + 1 : filepath;
}
