cmake_minimum_required(VERSION 3.14)
project(SpedImage LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Fix for libde265 using old CMake policies (< 3.5)
if(POLICY CMP0048)
    cmake_policy(SET CMP0048 NEW)
endif()
# Ensure newer policies are used
set(CMAKE_POLICY_VERSION_MINIMUM 3.5)


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
    DOWNLOAD_EXTRACT_TIMESTAMP true
)

# 2. Dear ImGui
FetchContent_Declare(
    imgui
    GIT_REPOSITORY https://github.com/ocornut/imgui.git
    GIT_TAG        docking
    DOWNLOAD_EXTRACT_TIMESTAMP true
)

# 3. ZLIB (Common dependency)
FetchContent_Declare(
    zlib
    GIT_REPOSITORY https://github.com/madler/zlib.git
    GIT_TAG        v1.3.1
    DOWNLOAD_EXTRACT_TIMESTAMP true
)

# 4. libde265 (HEIC Decoder)
FetchContent_Declare(
    libde265
    GIT_REPOSITORY https://github.com/strukturag/libde265.git
    GIT_TAG        v1.0.15
    DOWNLOAD_EXTRACT_TIMESTAMP true
)
# Disable strict warnings for dependencies that are noisy (especially libde265 on MSVC)
if(MSVC)
    add_compile_definitions(_CRT_SECURE_NO_WARNINGS) # Fix C4996 "unsafe" functions
    add_compile_options(/wd4309) # argument: truncation of constant value
    add_compile_options(/wd4244) # conversion: possible loss of data
    add_compile_options(/wd4267) # conversion: size_t to int
    add_compile_options(/wd4101) # unreferenced local variable
    add_compile_options(/wd4996) # deprecated functions (redundant with define but safe)
endif()
set(WARNINGS_AS_ERRORS OFF CACHE BOOL "" FORCE)

# Force disable Doxygen to prevent build failures (missing dot.exe)
set(DOXYGEN_EXECUTABLE "" CACHE FILEPATH "" FORCE)
set(DOXYGEN_FOUND FALSE CACHE BOOL "" FORCE)

# 5. libheif
FetchContent_Declare(
    libheif
    GIT_REPOSITORY https://github.com/strukturag/libheif.git
    GIT_TAG        v1.17.6
    DOWNLOAD_EXTRACT_TIMESTAMP true
)

# Populate dependencies to set up paths
FetchContent_MakeAvailable(glfw imgui zlib libde265)

# Ensure libde265 does NOT treat warnings as errors (it might override global settings)
if(TARGET libde265)
    target_compile_options(libde265 PRIVATE /WX-) 
endif()
if(TARGET encoder)
    target_compile_options(encoder PRIVATE /WX-)
endif()

# Configure libheif to find our fetched libde265
set(LIBDE265_FOUND ON CACHE BOOL "" FORCE)
# Include both Source (for headers) and Binary (for generated de265-version.h)
set(LIBDE265_INCLUDE_DIRS "${libde265_SOURCE_DIR}" "${libde265_BINARY_DIR}" CACHE PATH "" FORCE)
set(LIBDE265_LIBRARIES libde265 CACHE STRING "" FORCE) 

# Disable heavy dependencies and documentation for subprojects
set(BUILD_DOCS OFF CACHE BOOL "" FORCE)
set(BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(BUILD_TESTING OFF CACHE BOOL "" FORCE)
set(ENABLE_EXAMPLES OFF CACHE BOOL "" FORCE)
set(ENABLE_DOCUMENTATION OFF CACHE BOOL "" FORCE)
set(WITH_DOXYGEN OFF CACHE BOOL "" FORCE)

set(WITH_LIBDE265 ON CACHE BOOL "" FORCE)
set(WITH_X265 OFF CACHE BOOL "" FORCE)     
set(WITH_AOM_DECODER OFF CACHE BOOL "" FORCE) 
set(WITH_AOM_ENCODER OFF CACHE BOOL "" FORCE)
set(WITH_EXAMPLES OFF CACHE BOOL "" FORCE)
set(WITH_RAV1E OFF CACHE BOOL "" FORCE)
set(WITH_DAV1D OFF CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(libheif)

# 6. libspng (Linux PNG)
if(UNIX AND NOT APPLE)
    FetchContent_Declare(
        libspng
        GIT_REPOSITORY https://github.com/randy408/libspng.git
        GIT_TAG        v0.7.4
        DOWNLOAD_EXTRACT_TIMESTAMP true
    )
    FetchContent_MakeAvailable(libspng)
endif()

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

# Include directories for ImGui and libraries
target_include_directories(SpedImage PRIVATE
    ${imgui_SOURCE_DIR}
    ${imgui_SOURCE_DIR}/backends
    ${libheif_SOURCE_DIR} # Fix for libheif/heif.h
    ${libde265_SOURCE_DIR}
    ${libde265_BINARY_DIR}
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
