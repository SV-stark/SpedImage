#ifndef UTILS_H
#define UTILS_H

#include <stdarg.h>
#include <stddef.h>


extern bool g_verbose;

void log_info(const char *fmt, ...);

typedef struct {
  char **files;
  size_t count;
  size_t capacity;
} FileList;

void file_list_init(FileList *list);
void file_list_free(FileList *list);
bool file_list_scan_directory(FileList *list, const char *directory_path);
int file_list_find_index(FileList *list, const char *filename);

// Helper to get directory path from full file path
// Returns newly allocated string that must be freed
char *get_directory_from_path(const char *filepath);

// Helper to get filename from full file path
const char *get_filename_from_path(const char *filepath);

#endif
