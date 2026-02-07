#include "App.h"
#include <iostream>
#include <objbase.h>

int main(int argc, char **argv) {
  HRESULT hr = CoInitialize(NULL);
  if (FAILED(hr)) {
    std::cerr << "Failed to initialize COM library." << std::endl;
    return -1;
  }

  App app("SpedImage (C++20)", 1280, 720);
  app.Run();

  CoUninitialize();
  return 0;
}
