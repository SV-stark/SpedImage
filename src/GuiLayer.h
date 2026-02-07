#pragma once

class App;

class GuiLayer {
public:
  static void Init();
  static void Shutdown();
  static void Render(App *app);

  // Layout settings
  static bool ShowSidebar;
  static float SidebarWidth;
};
