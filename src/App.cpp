#include "App.h"
#include "Editor.h"
#include "GuiLayer.h"
#include "Image.h"
#include <GLFW/glfw3.h>
#include <backends/imgui_impl_glfw.h>
#include <backends/imgui_impl_opengl3.h>
#include <filesystem>
#include <imgui.h>
#include <stdio.h>

static void glfw_error_callback(int error, const char *description) {
  fprintf(stderr, "GLFW Error %d: %s\n", error, description);
}

App::App(const std::string &title, int width, int height)
    : m_Title(title), m_Width(width), m_Height(height) {
  InitWindow();
  InitImGui();
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
  // Update logic
}

void App::OnRender() {
  ImGui_ImplOpenGL3_NewFrame();
  ImGui_ImplGlfw_NewFrame();
  ImGui::NewFrame();

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
    index = (int)std::distance(files.begin(), it);
    index = (index + 1) % files.size();
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
    index = (int)std::distance(files.begin(), it);
    index = (index - 1 + files.size()) % files.size();
  }
  std::string parent =
      std::filesystem::path(m_CurrentPath).parent_path().string();
  if (parent.empty())
    parent = ".";
  LoadImage(parent + "/" + files[index]);
}

void App::SaveImage() {
  // TODO: Implement save
}

void App::RotateImage() {
  if (m_CurrentImage)
    Editor::Rotate(m_CurrentImage.get());
}

void App::CropImage() {
  // TODO: Implement crop
}
