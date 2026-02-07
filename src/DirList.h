#pragma once
#include <filesystem>
#include <string>
#include <vector>


class DirList {
public:
  void Scan(const std::string &path);
  const std::vector<std::string> &GetFiles() const { return m_Files; }

  // Helper to get only image files
  static bool IsImage(const std::string &filename);

private:
  std::vector<std::string> m_Files;
};
