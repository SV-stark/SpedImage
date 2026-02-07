#include "DirList.h"
#include <algorithm>
#include <set>

namespace fs = std::filesystem;

bool DirList::IsImage(const std::string &filename) {
  std::string ext = fs::path(filename).extension().string();
  std::transform(ext.begin(), ext.end(), ext.begin(), ::tolower);
  static const std::set<std::string> extensions = {
      ".jpg", ".jpeg", ".png", ".bmp", ".tga", ".gif", ".webp", ".svg"};
  return extensions.find(ext) != extensions.end();
}

void DirList::Scan(const std::string &path) {
  m_Files.clear();
  if (!fs::exists(path) || !fs::is_directory(path))
    return;

  for (const auto &entry : fs::directory_iterator(path)) {
    if (entry.is_regular_file()) {
      std::string filename = entry.path().filename().string();
      if (IsImage(filename)) {
        m_Files.push_back(filename);
      }
    }
  }
  std::sort(m_Files.begin(), m_Files.end());
}
