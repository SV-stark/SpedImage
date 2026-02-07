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
  std::shared_ptr<const class Editor> GetEditor() const { return m_Editor; }
  std::shared_ptr<class Editor> GetEditor() { return m_Editor; }

  // Crop State
  bool m_IsCropping = false;
  float m_ProposedCrop[4] = {0.0f, 0.0f, 1.0f, 1.0f};

  void ApplyCrop();
  void ResetCrop();
  void CancelCrop();

  // Accessors
  bool IsCropping() const { return m_IsCropping; }
  float *GetProposedCrop() { return m_ProposedCrop; }

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
  std::shared_ptr<class Editor> m_Editor; // GPU Renderer
  std::string m_CurrentPath;
  class DirList m_DirList;
  int m_CurrentFileIndex = -1;
};
