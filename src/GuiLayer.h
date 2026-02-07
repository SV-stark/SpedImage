#pragma once

#include <memory>

class App;
class Image;

class GuiLayer {
public:
  static void Init();
  static void Shutdown();
  static void Render(App *app);

  // Layout settings
  static bool ShowSidebar;
  static float SidebarWidth;

private:
  static void RenderStatusBar(App* app, std::shared_ptr<class Image> img, 
                              float sidebarWidth, float menuHeight, 
                              float viewportW, float viewportH);
};
