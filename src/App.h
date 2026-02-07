#pragma once

#include "DirList.h"
#include <memory>
#include <string>
#include <vector>


struct GLFWwindow;

class App {
public:
  App(const std::string &title, int width, int height);
  ~App();

  void Run();

  // UI callbacks
  void LoadImage(const std::string &path);
  void SaveImage();
  void NextImage();
  void PrevImage();
  void RotateImage();
  void CropImage(); // Enters crop mode or applies crop

  // Accessors
  std::shared_ptr<class Image> GetImage() const { return m_CurrentImage; }
  const class DirList &GetDirList() const { return m_DirList; }
  const std::string &GetCurrentPath() const { return m_CurrentPath; }

private:
  void InitWindow();
  void InitImGui();
  void Shutdown();

  void OnUpdate();
  void OnRender();

  GLFWwindow *m_Window = nullptr;
  std::string m_Title;
  int m_Width;
  int m_Height;
  bool m_Running = true;

  // State
  std::shared_ptr<class Image> m_CurrentImage;
  std::string m_CurrentPath;
  class DirList m_DirList;
  int m_CurrentFileIndex = -1;
};
