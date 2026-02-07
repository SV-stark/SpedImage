cmake_minimum_required(VERSION 3.14)
project(SpedImage LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Define sources
set(SOURCES
    src/main.cpp
    src/App.cpp
    src/GuiLayer.cpp
    src/Image.cpp
    src/Editor.cpp
)

# Headers
include_directories(include)

# ------------------------------------------------------------------------------
# Dependencies (FetchContent)
# ------------------------------------------------------------------------------
include(FetchContent)

# 1. GLFW
FetchContent_Declare(
    glfw
    GIT_REPOSITORY https://github.com/glfw/glfw.git
    GIT_TAG        3.4
)
FetchContent_MakeAvailable(glfw)

# 2. Dear ImGui
FetchContent_Declare(
    imgui
    GIT_REPOSITORY https://github.com/ocornut/imgui.git
    GIT_TAG        docking
)
FetchContent_MakeAvailable(imgui)

# ------------------------------------------------------------------------------
# Executable
# ------------------------------------------------------------------------------

# Add backend sources
set(IMGUI_SOURCES
    ${imgui_SOURCE_DIR}/backends/imgui_impl_glfw.cpp
    ${imgui_SOURCE_DIR}/backends/imgui_impl_opengl3.cpp
)

add_executable(SpedImage ${SOURCES} ${IMGUI_SOURCES})

# Include directories for ImGui backends
target_include_directories(SpedImage PRIVATE
    ${imgui_SOURCE_DIR}
    ${imgui_SOURCE_DIR}/backends
)

# Link libraries
target_link_libraries(SpedImage PRIVATE
    glfw
    imgui
)

# Platform specific linking
if(WIN32)
    target_link_libraries(SpedImage PRIVATE opengl32)
elseif(UNIX AND NOT APPLE)
    target_link_libraries(SpedImage PRIVATE GL)
endif()

# Copy assets to build directory
add_custom_command(TARGET SpedImage POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_directory
    ${CMAKE_SOURCE_DIR}/assets ${CMAKE_BINARY_DIR}/assets
    COMMENT "Copying assets..."
)
