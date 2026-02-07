#include "App.h"
#include "Editor.h"
#include "GuiLayer.h"
#include "Image.h"
#include <GLFW/glfw3.h>
#include <backends/imgui_impl_glfw.h>
#include <backends/imgui_impl_opengl3.h>
#include <filesystem>
#include <imgui.h>
#include <iostream>
#include <stb_image_write.h>
#include <stdio.h>

static void glfw_error_callback(int error, const char *description) {
  fprintf(stderr, "GLFW Error %d: %s\n", error, description);
}

App::App(const std::string &title, int width, int height)
    : m_Title(title), m_Width(width), m_Height(height) {
  InitWindow();
  InitImGui();

  // Initialize GPU Editor
  m_Editor = std::make_shared<Editor>();
  m_Editor->Init();

  GuiLayer::Init();

  // Default scan current directory
  auto currentPath = std::filesystem::current_path();
  m_DirList.Scan(currentPath.string());
}

App::~App() { Shutdown(); }

void App::InitWindow() {
  glfwSetErrorCallback(glfw_error_callback);
  if (!glfwInit())
    return;

  const char *glsl_version = "#version 130";
  glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
  glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
  // glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

  m_Window = glfwCreateWindow(m_Width, m_Height, m_Title.c_str(), NULL, NULL);
  if (m_Window == NULL)
    return;

  glfwMakeContextCurrent(m_Window);
  glfwSwapInterval(1); // Enable vsync
}

void App::InitImGui() {
  IMGUI_CHECKVERSION();
  ImGui::CreateContext();
  ImGuiIO &io = ImGui::GetIO();
  (void)io;
  io.ConfigFlags |=
      ImGuiConfigFlags_NavEnableKeyboard;           // Enable Keyboard Controls
  io.ConfigFlags |= ImGuiConfigFlags_DockingEnable; // Enable Docking

  ImGui::StyleColorsDark();

  ImGui_ImplGlfw_InitForOpenGL(m_Window, true);
  ImGui_ImplOpenGL3_Init("#version 130");
}

void App::Shutdown() {
  GuiLayer::Shutdown();
  ImGui_ImplOpenGL3_Shutdown();
  ImGui_ImplGlfw_Shutdown();
  ImGui::DestroyContext();

  if (m_Window) {
    glfwDestroyWindow(m_Window);
    m_Window = nullptr;
  }
  glfwTerminate();
}

void App::Run() {
  if (!m_Window)
    return;

  while (!glfwWindowShouldClose(m_Window)) {
    glfwPollEvents();
    OnUpdate();
    OnRender();
  }
}

void App::OnUpdate() {
  // Shortcuts
  ImGuiIO &io = ImGui::GetIO();
  if (io.KeyCtrl && ImGui::IsKeyPressed(ImGuiKey_S, false)) {
    SaveImage();
  }
}

void App::OnRender() {
  ImGui_ImplOpenGL3_NewFrame();
  ImGui_ImplGlfw_NewFrame();
  ImGui::NewFrame();

  // 1. Render Image to FBO using Shader (GPU Adjustment)
  if (m_CurrentImage) {
    m_Editor->Render(m_CurrentImage);
  }

  // 2. Render UI
  GuiLayer::Render(this);

  // -------------------------------------------------------------------------
  // Rendering
  // -------------------------------------------------------------------------
  ImGui::Render();
  int display_w, display_h;
  glfwGetFramebufferSize(m_Window, &display_w, &display_h);
  glViewport(0, 0, display_w, display_h);
  glClearColor(0.1f, 0.1f, 0.11f, 1.0f);
  glClear(GL_COLOR_BUFFER_BIT);
  ImGui_ImplOpenGL3_RenderDrawData(ImGui::GetDrawData());

  glfwSwapBuffers(m_Window);
}

void App::LoadImage(const std::string &path) {
  if (!std::filesystem::exists(path))
    return;
  auto img = std::make_shared<Image>(path);
  if (img->Load()) {
    m_CurrentImage = img;
    m_CurrentPath = path;

    std::string title =
        "SpedImage - " + std::filesystem::path(path).filename().string();
    glfwSetWindowTitle(m_Window, title.c_str());

    std::string parent = std::filesystem::path(path).parent_path().string();
    if (parent.empty())
      parent = ".";
    m_DirList.Scan(parent);
  }
}

void App::NextImage() {
  const auto &files = m_DirList.GetFiles();
  if (files.empty())
    return;
  std::string currentFile =
      std::filesystem::path(m_CurrentPath).filename().string();
  auto it = std::find(files.begin(), files.end(), currentFile);
  int index = 0;
  if (it != files.end()) {
    index = static_cast<int>(std::distance(files.begin(), it));
    index = (index + 1) % static_cast<int>(files.size());
  }
  std::string parent =
      std::filesystem::path(m_CurrentPath).parent_path().string();
  if (parent.empty())
    parent = ".";
  LoadImage(parent + "/" + files[index]);
}

void App::PrevImage() {
  const auto &files = m_DirList.GetFiles();
  if (files.empty())
    return;
  std::string currentFile =
      std::filesystem::path(m_CurrentPath).filename().string();
  auto it = std::find(files.begin(), files.end(), currentFile);
  int index = 0;
  if (it != files.end()) {
    index = static_cast<int>(std::distance(files.begin(), it));
    index = (index - 1 + static_cast<int>(files.size())) %
            static_cast<int>(files.size());
  }
  std::string parent =
      std::filesystem::path(m_CurrentPath).parent_path().string();
  if (parent.empty())
    parent = ".";
  LoadImage(parent + "/" + files[index]);
}

void App::SaveImage() {
  if (!m_CurrentImage || !m_Editor)
    return;

  // 1. Get Pixels from GPU
  std::vector<uint8_t> pixels = m_Editor->GetPixels();
  int w = m_Editor->GetWidth(); // Use Editor's dimensions (might differ from
                                // original if resized/cropped)
  int h = m_Editor->GetHeight();

  if (pixels.empty() || w <= 0 || h <= 0)
    return;

  // 2. Generate Filename (e.g. image_edited.png)
  std::filesystem::path srcPath(m_CurrentPath);
  std::string stem = srcPath.stem().string();
  std::string ext = ".png"; // Force PNG for now for lossless save
  std::filesystem::path destPath =
      srcPath.parent_path() / (stem + "_edited" + ext);

  // 3. Save
  // Flip vertically because GL is bottom-left
  stbi_flip_vertically_on_write(1);
  if (stbi_write_png(destPath.string().c_str(), w, h, 4, pixels.data(),
                     w * 4)) {
    std::cout << "Saved to " << destPath << std::endl;

    // Optional: Load the new image?
    // LoadImage(destPath.string());
  } else {
    std::cerr << "Failed to save to " << destPath << std::endl;
  }
}

void App::RotateImage() {
  if (m_Editor) {
    // Rotate 90 degrees (PI/2)
    m_Editor->Rotation += 1.57079632679f; // 90 deg in radians
    // Keep it normalized if needed, or shader handles it
  }
}

void App::ApplyCrop() {
  if (m_Editor) {
    for (int i = 0; i < 4; i++)
      m_Editor->CropRect[i] = m_ProposedCrop[i];
  }
  m_IsCropping = false;
}

void App::ResetCrop() {
  if (m_Editor) {
    m_Editor->CropRect[0] = 0.0f;
    m_Editor->CropRect[1] = 0.0f;
    m_Editor->CropRect[2] = 1.0f;
    m_Editor->CropRect[3] = 1.0f;
  }
  m_IsCropping = false;
  m_ProposedCrop[0] = 0.0f;
  m_ProposedCrop[1] = 0.0f;
  m_ProposedCrop[2] = 1.0f;
  m_ProposedCrop[3] = 1.0f;
}

void App::CancelCrop() { m_IsCropping = false; }

void App::CropImage() {
  m_IsCropping = !m_IsCropping;
  if (m_IsCropping && m_Editor) {
    // Initialize proposed with current
    for (int i = 0; i < 4; i++)
      m_ProposedCrop[i] = m_Editor->CropRect[i];
  }
}
