#include "GuiLayer.h"
#include "App.h"
#include "DirList.h"
#include "Image.h"
#include <GLFW/glfw3.h> // For keys
#include <algorithm>
#include <filesystem>
#include <imgui.h>
#include <string>

bool GuiLayer::ShowSidebar = true;
float GuiLayer::SidebarWidth = 280.0f;

// Modern color palette
namespace Colors {
// Background colors
constexpr ImVec4 BackgroundDark = ImVec4(0.08f, 0.08f, 0.10f, 1.0f);
constexpr ImVec4 BackgroundMedium = ImVec4(0.12f, 0.12f, 0.15f, 1.0f);
constexpr ImVec4 BackgroundLight = ImVec4(0.16f, 0.16f, 0.20f, 1.0f);

// Accent colors
constexpr ImVec4 AccentBlue = ImVec4(0.29f, 0.56f, 0.85f, 1.0f);
constexpr ImVec4 AccentHover = ImVec4(0.39f, 0.66f, 0.95f, 1.0f);
constexpr ImVec4 AccentActive = ImVec4(0.20f, 0.46f, 0.75f, 1.0f);

// Text colors
constexpr ImVec4 TextPrimary = ImVec4(0.95f, 0.95f, 0.97f, 1.0f);
constexpr ImVec4 TextSecondary = ImVec4(0.60f, 0.60f, 0.65f, 1.0f);
constexpr ImVec4 TextMuted = ImVec4(0.40f, 0.40f, 0.45f, 1.0f);

// Border and separator
constexpr ImVec4 Border = ImVec4(0.20f, 0.20f, 0.25f, 1.0f);
constexpr ImVec4 Separator = ImVec4(0.15f, 0.15f, 0.18f, 1.0f);
} // namespace Colors

void GuiLayer::Init() {
  ImGui::StyleColorsDark();
  ImGuiStyle &style = ImGui::GetStyle();

  // Modern rounded design
  style.WindowRounding = 8.0f;
  style.ChildRounding = 8.0f;
  style.FrameRounding = 6.0f;
  style.PopupRounding = 8.0f;
  style.ScrollbarRounding = 6.0f;
  style.GrabRounding = 4.0f;
  style.TabRounding = 6.0f;

  // Spacing
  style.WindowPadding = ImVec2(16.0f, 16.0f);
  style.FramePadding = ImVec2(12.0f, 8.0f);
  style.ItemSpacing = ImVec2(8.0f, 6.0f);
  style.ItemInnerSpacing = ImVec2(8.0f, 6.0f);
  style.IndentSpacing = 16.0f;

  // Borders
  style.WindowBorderSize = 0.0f;
  style.ChildBorderSize = 0.0f;
  style.PopupBorderSize = 1.0f;
  style.FrameBorderSize = 0.0f;
  style.TabBorderSize = 0.0f;

  // Scrollbar
  style.ScrollbarSize = 8.0f;

  // Apply color scheme
  ImVec4 *colors = style.Colors;
  colors[ImGuiCol_WindowBg] = Colors::BackgroundDark;
  colors[ImGuiCol_ChildBg] = Colors::BackgroundMedium;
  colors[ImGuiCol_PopupBg] = Colors::BackgroundLight;
  colors[ImGuiCol_Border] = Colors::Border;
  colors[ImGuiCol_Separator] = Colors::Separator;
  colors[ImGuiCol_SeparatorActive] = Colors::AccentBlue;
  colors[ImGuiCol_SeparatorHovered] = Colors::AccentHover;
  colors[ImGuiCol_FrameBg] = Colors::BackgroundLight;
  colors[ImGuiCol_FrameBgHovered] = ImVec4(0.20f, 0.20f, 0.25f, 1.0f);
  colors[ImGuiCol_FrameBgActive] = ImVec4(0.24f, 0.24f, 0.30f, 1.0f);
  colors[ImGuiCol_TitleBg] = Colors::BackgroundMedium;
  colors[ImGuiCol_TitleBgActive] = Colors::BackgroundLight;
  colors[ImGuiCol_MenuBarBg] = Colors::BackgroundMedium;
  colors[ImGuiCol_ScrollbarBg] = Colors::BackgroundDark;
  colors[ImGuiCol_ScrollbarGrab] = ImVec4(0.30f, 0.30f, 0.35f, 1.0f);
  colors[ImGuiCol_ScrollbarGrabHovered] = ImVec4(0.40f, 0.40f, 0.45f, 1.0f);
  colors[ImGuiCol_ScrollbarGrabActive] = Colors::AccentBlue;
  colors[ImGuiCol_CheckMark] = Colors::AccentBlue;
  colors[ImGuiCol_SliderGrab] = Colors::AccentBlue;
  colors[ImGuiCol_SliderGrabActive] = Colors::AccentActive;
  colors[ImGuiCol_Button] = Colors::BackgroundLight;
  colors[ImGuiCol_ButtonHovered] = ImVec4(0.24f, 0.24f, 0.30f, 1.0f);
  colors[ImGuiCol_ButtonActive] = ImVec4(0.29f, 0.29f, 0.36f, 1.0f);
  colors[ImGuiCol_Header] = Colors::BackgroundLight;
  colors[ImGuiCol_HeaderHovered] = ImVec4(0.24f, 0.24f, 0.30f, 1.0f);
  colors[ImGuiCol_HeaderActive] = Colors::AccentBlue;
  colors[ImGuiCol_Separator] = Colors::Separator;
  colors[ImGuiCol_SeparatorHovered] = Colors::AccentHover;
  colors[ImGuiCol_SeparatorActive] = Colors::AccentBlue;
  colors[ImGuiCol_ResizeGrip] = Colors::AccentBlue;
  colors[ImGuiCol_ResizeGripHovered] = Colors::AccentHover;
  colors[ImGuiCol_ResizeGripActive] = Colors::AccentActive;
  colors[ImGuiCol_Tab] = Colors::BackgroundMedium;
  colors[ImGuiCol_TabHovered] = Colors::BackgroundLight;
  colors[ImGuiCol_TabActive] = Colors::AccentBlue;
  colors[ImGuiCol_Text] = Colors::TextPrimary;
  colors[ImGuiCol_TextDisabled] = Colors::TextMuted;
  colors[ImGuiCol_TextSelectedBg] = ImVec4(0.29f, 0.56f, 0.85f, 0.35f);
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

  // Modern Top Bar
  float menuBarHeight = 0.0f;
  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2(16.0f, 12.0f));
  if (ImGui::BeginMainMenuBar()) {
    menuBarHeight = ImGui::GetWindowHeight();

    // App title/logo area
    ImGui::TextColored(Colors::AccentBlue, "SpedImage");
    ImGui::SameLine();
    ImGui::TextColored(Colors::TextMuted, "|");
    ImGui::SameLine();

    if (ImGui::BeginMenu("File")) {
      if (ImGui::MenuItem("Open...", "Ctrl+O")) {
        // TODO: Open Dialog
      }
      ImGui::Separator();
      if (ImGui::MenuItem("Save", "Ctrl+S"))
        app->SaveImage();
      ImGui::Separator();
      if (ImGui::MenuItem("Exit", "Alt+F4")) {
        // Trigger app close
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

      ImGui::Separator();
      ImGui::TextDisabled("Adjustments");

      auto editor = app->GetEditor();
      if (editor) {
        ImGui::SliderFloat("Brightness", &editor->Brightness, 0.0f, 2.0f);
        ImGui::SliderFloat("Contrast", &editor->Contrast, 0.0f, 2.0f);
        ImGui::SliderFloat("Saturation", &editor->Saturation, 0.0f, 2.0f);

        if (ImGui::Button("Reset")) {
          editor->Brightness = 1.0f;
          editor->Contrast = 1.0f;
          editor->Saturation = 1.0f;
          editor->Rotation = 0.0f;
        }
      }

      ImGui::EndMenu();
    }

    // Right-aligned info
    auto &io = ImGui::GetIO();
    std::string info = std::to_string(static_cast<int>(io.Framerate)) + " FPS";
    float infoWidth = ImGui::CalcTextSize(info.c_str()).x;
    ImGui::SameLine(io.DisplaySize.x - infoWidth - 20.0f);
    ImGui::TextColored(Colors::TextMuted, "%s", info.c_str());

    ImGui::EndMainMenuBar();
  }
  ImGui::PopStyleVar();

  // Modern Sidebar
  float sidebarRealWidth = 0.0f;
  if (ShowSidebar) {
    ImGui::SetNextWindowPos(ImVec2(0, menuBarHeight));
    ImGui::SetNextWindowSize(
        ImVec2(SidebarWidth, ImGui::GetIO().DisplaySize.y - menuBarHeight));

    ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2(12.0f, 16.0f));
    ImGui::Begin("Sidebar", &ShowSidebar,
                 ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoResize |
                     ImGuiWindowFlags_NoCollapse | ImGuiWindowFlags_NoTitleBar);
    ImGui::PopStyleVar();

    // Sidebar Header
    ImGui::TextColored(Colors::TextSecondary, "FILES");
    ImGui::Spacing();
    ImGui::Separator();
    ImGui::Spacing();

    const auto &files = app->GetDirList().GetFiles();
    std::string currentPath = app->GetCurrentPath();
    std::string currentName =
        std::filesystem::path(currentPath).filename().string();

    // File list with modern styling
    ImGui::BeginChild("FileList", ImVec2(0, -30), false);

    int index = 0;
    for (const auto &file : files) {
      bool isSelected = (currentName == file);

      // Custom styled selectable
      ImGui::PushID(index++);

      // File icon and name
      ImVec2 cursorPos = ImGui::GetCursorScreenPos();
      ImVec2 itemSize = ImVec2(ImGui::GetContentRegionAvail().x, 36.0f);

      // Background hover/selection effect
      ImDrawList *drawList = ImGui::GetWindowDrawList();
      bool hovered = ImGui::IsMouseHoveringRect(
          cursorPos,
          ImVec2(cursorPos.x + itemSize.x, cursorPos.y + itemSize.y));

      if (isSelected) {
        drawList->AddRectFilled(
            cursorPos,
            ImVec2(cursorPos.x + itemSize.x, cursorPos.y + itemSize.y),
            ImGui::ColorConvertFloat4ToU32(ImVec4(0.29f, 0.56f, 0.85f, 0.25f)),
            6.0f);
        drawList->AddRect(
            cursorPos,
            ImVec2(cursorPos.x + itemSize.x, cursorPos.y + itemSize.y),
            ImGui::ColorConvertFloat4ToU32(Colors::AccentBlue), 6.0f, 0, 1.5f);
      } else if (hovered) {
        drawList->AddRectFilled(
            cursorPos,
            ImVec2(cursorPos.x + itemSize.x, cursorPos.y + itemSize.y),
            ImGui::ColorConvertFloat4ToU32(ImVec4(1.0f, 1.0f, 1.0f, 0.05f)),
            6.0f);
      }

      // Invisible button for click detection
      ImGui::InvisibleButton("file", itemSize);
      if (ImGui::IsItemClicked()) {
        std::string parent =
            std::filesystem::path(currentPath).parent_path().string();
        if (parent.empty())
          parent = ".";
        app->LoadImage(parent + "/" + file);
      }

      // Draw file icon (simple square with rounded corners)
      ImVec2 iconPos = ImVec2(cursorPos.x + 8.0f, cursorPos.y + 6.0f);
      ImVec2 iconSize = ImVec2(24.0f, 24.0f);
      ImU32 iconColor = ImGui::ColorConvertFloat4ToU32(
          isSelected ? Colors::AccentBlue : Colors::TextMuted);
      drawList->AddRectFilled(
          iconPos, ImVec2(iconPos.x + iconSize.x, iconPos.y + iconSize.y),
          iconColor, 4.0f);

      // File name
      ImVec2 textPos =
          ImVec2(cursorPos.x + 40.0f,
                 cursorPos.y + (itemSize.y - ImGui::GetFontSize()) * 0.5f);
      ImU32 textColor = ImGui::ColorConvertFloat4ToU32(
          isSelected ? Colors::TextPrimary : Colors::TextSecondary);

      // Truncate long filenames
      std::string displayName = file;
      float maxTextWidth = itemSize.x - 50.0f;
      float textWidth = ImGui::CalcTextSize(displayName.c_str()).x;
      if (textWidth > maxTextWidth) {
        while (displayName.length() > 3 &&
               ImGui::CalcTextSize((displayName + "...").c_str()).x >
                   maxTextWidth) {
          displayName.pop_back();
        }
        displayName += "...";
      }

      drawList->AddText(textPos, textColor, displayName.c_str());

      ImGui::PopID();
      ImGui::Spacing();
    }

    ImGui::EndChild();

    // File count at bottom
    ImGui::Separator();
    ImGui::Spacing();
    std::string countText = std::to_string(files.size()) + " files";
    ImGui::TextColored(Colors::TextMuted, "%s", countText.c_str());

    sidebarRealWidth = ImGui::GetWindowWidth();
    ImGui::End();
  }

  // Image Viewport
  ImGui::SetNextWindowPos(ImVec2(sidebarRealWidth, menuBarHeight));
  ImGui::SetNextWindowSize(
      ImVec2(ImGui::GetIO().DisplaySize.x - sidebarRealWidth,
             ImGui::GetIO().DisplaySize.y - menuBarHeight));
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
    // Use the Editor's FBO texture which has shaders applied
    ImGui::Image(app->GetEditor()->GetTextureID(), ImVec2(drawW, drawH));

    // Interactive Crop Tool
    if (app->IsCropping()) {
      ImVec2 imgMin = ImGui::GetItemRectMin();
      ImVec2 imgMax = ImGui::GetItemRectMax();

      float *crop = app->GetProposedCrop(); // x, y, w, h

      // Convert normalized UV to Screen
      ImVec2 cropMin =
          ImVec2(imgMin.x + crop[0] * drawW, imgMin.y + crop[1] * drawH);
      ImVec2 cropMax =
          ImVec2(cropMin.x + crop[2] * drawW, cropMin.y + crop[3] * drawH);

      ImDrawList *dl = ImGui::GetWindowDrawList();

      // Dim outside area
      // Top
      dl->AddRectFilled(imgMin, ImVec2(imgMax.x, cropMin.y),
                        IM_COL32(0, 0, 0, 160));
      // Bottom
      dl->AddRectFilled(ImVec2(imgMin.x, cropMax.y), imgMax,
                        IM_COL32(0, 0, 0, 160));
      // Left
      dl->AddRectFilled(ImVec2(imgMin.x, cropMin.y),
                        ImVec2(cropMin.x, cropMax.y), IM_COL32(0, 0, 0, 160));
      // Right
      dl->AddRectFilled(ImVec2(cropMax.x, cropMin.y),
                        ImVec2(imgMax.x, cropMax.y), IM_COL32(0, 0, 0, 160));

      // Draw selection border
      dl->AddRect(cropMin, cropMax, IM_COL32(255, 200, 50, 255), 0.0f, 0, 2.0f);

      // Invisible button for interaction
      ImGui::SetCursorPos(ImVec2(x, y));
      ImGui::InvisibleButton("CropInput", ImVec2(drawW, drawH));

      static ImVec2 startUV;
      if (ImGui::IsItemActive()) {
        ImVec2 mousePos = ImGui::GetMousePos();
        float u = (mousePos.x - imgMin.x) / drawW;
        float v = (mousePos.y - imgMin.y) / drawH;
        u = std::max(0.0f, std::min(u, 1.0f));
        v = std::max(0.0f, std::min(v, 1.0f));

        if (ImGui::IsItemClicked()) {
          startUV = ImVec2(u, v);
        }

        float minU = std::min(startUV.x, u);
        float minV = std::min(startUV.y, v);
        float w = std::abs(u - startUV.x);
        float h = std::abs(v - startUV.y);

        if (w > 0.0f && h > 0.0f) {
          crop[0] = minU;
          crop[1] = minV;
          crop[2] = w;
          crop[3] = h;
        }
      }

      // Apply/Cancel Box at top-center of viewport
      ImGui::SetCursorPos(ImVec2(x + drawW * 0.5f - 80, y + 20));
      if (ImGui::Button("Cancel", ImVec2(70, 30)))
        app->CancelCrop();
      ImGui::SameLine();
      if (ImGui::Button("Apply", ImVec2(70, 30)))
        app->ApplyCrop();
    }

    // Status Bar Overlay (Bottom)
    RenderStatusBar(app, img, sidebarRealWidth, menuBarHeight, availW, availH);

  } else {
    // Empty state
    auto windowSize = ImGui::GetWindowSize();
    const char *title = "SpedImage";
    const char *subtitle = "Drop an image or press Ctrl+O to open";

    ImVec2 titleSize = ImGui::CalcTextSize(title);
    ImVec2 subtitleSize = ImGui::CalcTextSize(subtitle);

    float centerX = windowSize.x * 0.5f;
    float centerY = windowSize.y * 0.5f;

    ImGui::SetCursorPos(ImVec2(centerX - titleSize.x * 0.5f, centerY - 30));
    ImGui::TextColored(Colors::AccentBlue, "%s", title);

    ImGui::SetCursorPos(ImVec2(centerX - subtitleSize.x * 0.5f, centerY));
    ImGui::TextColored(Colors::TextMuted, "%s", subtitle);
  }

  ImGui::End();
  ImGui::PopStyleVar();
}

void GuiLayer::RenderStatusBar(App *app, std::shared_ptr<class Image> img,
                               float sidebarWidth, float menuHeight,
                               float viewportW, float viewportH) {
  if (!img)
    return;

  // Status bar at bottom of viewport
  float statusBarHeight = 32.0f;
  float statusBarY = ImGui::GetIO().DisplaySize.y - statusBarHeight - 8.0f;
  float statusBarX = sidebarWidth + 16.0f;
  float statusBarW = viewportW - 32.0f;

  ImDrawList *drawList = ImGui::GetForegroundDrawList();
  ImVec2 pos = ImVec2(statusBarX, statusBarY);
  ImVec2 size = ImVec2(statusBarW, statusBarHeight);

  // Background
  drawList->AddRectFilled(
      pos, ImVec2(pos.x + size.x, pos.y + size.y),
      ImGui::ColorConvertFloat4ToU32(ImVec4(0.08f, 0.08f, 0.10f, 0.90f)), 8.0f);
  drawList->AddRect(pos, ImVec2(pos.x + size.x, pos.y + size.y),
                    ImGui::ColorConvertFloat4ToU32(Colors::Border), 8.0f, 0,
                    1.0f);

  // Info text
  std::string info = std::to_string(img->GetWidth()) + "x" +
                     std::to_string(img->GetHeight()) + " px";
  ImVec2 textPos =
      ImVec2(pos.x + 16.0f, pos.y + (size.y - ImGui::GetFontSize()) * 0.5f);
  drawList->AddText(textPos,
                    ImGui::ColorConvertFloat4ToU32(Colors::TextSecondary),
                    info.c_str());
}
