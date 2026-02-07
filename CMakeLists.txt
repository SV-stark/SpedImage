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
    src/DirList.cpp
    src/stb_impl.cpp
)

if(WIN32)
    list(APPEND SOURCES src/ImageBackend_Win32.cpp)
endif()

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

# 3. libde265 (HEIC Decoder)
FetchContent_Declare(
    libde265
    GIT_REPOSITORY https://github.com/strukturag/libde265.git
    GIT_TAG        v1.0.15
)
# Disable tools/examples to speed up build
set(ENABLE_DE265_ENCODER OFF CACHE BOOL "" FORCE)
set(ENABLE_DE265_SHERLOCK265 OFF CACHE BOOL "" FORCE)
set(ENABLE_DE265_DEC265 OFF CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(libde265)

# 4. libheif
FetchContent_Declare(
    libheif
    GIT_REPOSITORY https://github.com/strukturag/libheif.git
    GIT_TAG        v1.17.6
)
set(WITH_LIBDE265 ON CACHE BOOL "" FORCE)
set(WITH_X265 OFF CACHE BOOL "" FORCE)     # Encoder
set(WITH_AOM_DECODER OFF CACHE BOOL "" FORCE) # AVIF (for now)
set(WITH_AOM_ENCODER OFF CACHE BOOL "" FORCE)
set(WITH_EXAMPLES OFF CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(libheif)

# 5. libspng (Linux PNG)
FetchContent_Declare(
    libspng
    GIT_REPOSITORY https://github.com/randy408/libspng.git
    GIT_TAG        v0.7.4
)
# We can fetch it on all platforms, but only link on Linux
FetchContent_MakeAvailable(libspng)

# ------------------------------------------------------------------------------
# Executable
# ------------------------------------------------------------------------------

# Add ImGui core and backend sources
set(IMGUI_SOURCES
    ${imgui_SOURCE_DIR}/imgui.cpp
    ${imgui_SOURCE_DIR}/imgui_draw.cpp
    ${imgui_SOURCE_DIR}/imgui_tables.cpp
    ${imgui_SOURCE_DIR}/imgui_widgets.cpp
    ${imgui_SOURCE_DIR}/backends/imgui_impl_glfw.cpp
    ${imgui_SOURCE_DIR}/backends/imgui_impl_opengl3.cpp
)

add_executable(SpedImage ${SOURCES} ${IMGUI_SOURCES})

# Include directories for ImGui
target_include_directories(SpedImage PRIVATE
    ${imgui_SOURCE_DIR}
    ${imgui_SOURCE_DIR}/backends
)

# Link libraries
target_link_libraries(SpedImage PRIVATE
    glfw
    libheif
)

# Platform specific linking
if(WIN32)
    target_link_libraries(SpedImage PRIVATE opengl32 windowscodecs shlwapi)
elseif(UNIX AND NOT APPLE)
    find_package(JPEG REQUIRED)
    # find_package(PNG) # Replaced by libspng
    target_link_libraries(SpedImage PRIVATE GL JPEG::JPEG spng_static)
    target_sources(SpedImage PRIVATE src/ImageBackend_Linux.cpp)
endif()

# Copy assets to build directory (if assets exist)
if(EXISTS ${CMAKE_SOURCE_DIR}/assets)
    add_custom_command(TARGET SpedImage POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_directory
        ${CMAKE_SOURCE_DIR}/assets ${CMAKE_BINARY_DIR}/assets
        COMMENT "Copying assets..."
    )
endif()
