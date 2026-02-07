#include "GuiLayer.h"
#include "App.h"
#include "DirList.h"
#include "Image.h"
#include <GLFW/glfw3.h> // For keys
#include <filesystem>
#include <imgui.h>
#include <string>


bool GuiLayer::ShowSidebar = true;
float GuiLayer::SidebarWidth = 250.0f;

void GuiLayer::Init() {
  ImGui::StyleColorsDark();
  ImGuiStyle &style = ImGui::GetStyle();
  style.WindowRounding = 4.0f;
  style.Colors[ImGuiCol_WindowBg] = ImVec4(0.1f, 0.1f, 0.13f, 1.0f);
}

void GuiLayer::Shutdown() {}

void GuiLayer::Render(App *app) {
  if (!app)
    return;

  // Shortcuts
  if (ImGui::IsKeyPressed(ImGuiKey_RightArrow))
    app->NextImage();
  if (ImGui::IsKeyPressed(ImGuiKey_LeftArrow))
    app->PrevImage();
  if (ImGui::IsKeyPressed(ImGuiKey_R))
    app->RotateImage();

  // Main Menu
  if (ImGui::BeginMainMenuBar()) {
    if (ImGui::BeginMenu("File")) {
      if (ImGui::MenuItem("Open...")) {
        // TODO: Open Dialog
      }
      if (ImGui::MenuItem("Save", "Ctrl+S"))
        app->SaveImage();
      if (ImGui::MenuItem("Exit", "Alt+F4")) {
        // Trigger app close
        // app->Close(); // Need to implement
      }
      ImGui::EndMenu();
    }
    if (ImGui::BeginMenu("View")) {
      ImGui::MenuItem("Sidebar", "F1", &ShowSidebar);
      ImGui::EndMenu();
    }
    if (ImGui::BeginMenu("Image")) {
      if (ImGui::MenuItem("Rotate 90 deg", "R"))
        app->RotateImage();
      if (ImGui::MenuItem("Crop", "C"))
        app->CropImage();
      ImGui::EndMenu();
    }
    ImGui::EndMainMenuBar();
  }

  // Sidebar
  float sidebarRealWidth = 0.0f;
  if (ShowSidebar) {
    ImGui::SetNextWindowPos(ImVec2(0, ImGui::GetFrameHeight()));
    ImGui::SetNextWindowSize(ImVec2(SidebarWidth, ImGui::GetIO().DisplaySize.y -
                                                      ImGui::GetFrameHeight()));
    ImGui::Begin("Sidebar", &ShowSidebar,
                 ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoResize |
                     ImGuiWindowFlags_NoCollapse | ImGuiWindowFlags_NoTitleBar);

    const auto &files = app->GetDirList().GetFiles();
    std::string currentPath = app->GetCurrentPath();
    std::string currentName =
        std::filesystem::path(currentPath).filename().string();

    for (const auto &file : files) {
      bool isSelected = (currentName == file);
      if (ImGui::Selectable(file.c_str(), isSelected)) {
        std::string parent =
            std::filesystem::path(currentPath).parent_path().string();
        if (parent.empty())
          parent = ".";
        app->LoadImage(parent + "/" + file);
      }
      if (isSelected)
        ImGui::SetItemDefaultFocus();
    }

    sidebarRealWidth = ImGui::GetWindowWidth();
    ImGui::End();
  }

  // Image Viewport
  ImGui::SetNextWindowPos(ImVec2(sidebarRealWidth, ImGui::GetFrameHeight()));
  ImGui::SetNextWindowSize(
      ImVec2(ImGui::GetIO().DisplaySize.x - sidebarRealWidth,
             ImGui::GetIO().DisplaySize.y - ImGui::GetFrameHeight()));
  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2(0, 0));
  ImGui::Begin("Viewport", nullptr,
               ImGuiWindowFlags_NoDecoration | ImGuiWindowFlags_NoMove |
                   ImGuiWindowFlags_NoResize |
                   ImGuiWindowFlags_NoBringToFrontOnFocus);

  auto img = app->GetImage();
  if (img) {
    float availW = ImGui::GetContentRegionAvail().x;
    float availH = ImGui::GetContentRegionAvail().y;
    float imgAspect = (float)img->GetWidth() / img->GetHeight();
    float viewAspect = availW / availH;

    float drawW = availW;
    float drawH = availW / imgAspect;
    if (drawH > availH) {
      drawH = availH;
      drawW = availH * imgAspect;
    }

    float x = (availW - drawW) * 0.5f;
    float y = (availH - drawH) * 0.5f;

    ImGui::SetCursorPos(ImVec2(x, y));
    // Cast to void* for ImTextureID
    ImGui::Image((void *)(intptr_t)img->GetTextureID(), ImVec2(drawW, drawH));
  } else {
    const char *text = "Drop an image to start";
    auto windowSize = ImGui::GetWindowSize();
    auto textSize = ImGui::CalcTextSize(text);
    ImGui::SetCursorPos(ImVec2((windowSize.x - textSize.x) * 0.5f,
                               (windowSize.y - textSize.y) * 0.5f));
    ImGui::Text("%s", text);
  }

  ImGui::End();
  ImGui::PopStyleVar();
}
