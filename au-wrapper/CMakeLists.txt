# Contrapunk AU (Audio Unit) Wrapper
#
# Wraps the pre-built CLAP plugin into an AUv2 .component using clap-wrapper.
# The CLAP is built by nih-plug (cargo xtask bundle), then this CMake project
# embeds it inside an AUv2 bundle that Logic Pro / GarageBand / any AU host can load.
#
# Usage:
#   1. cargo xtask bundle contrapunk_plugin --release
#   2. cd au-wrapper && mkdir -p build && cd build
#   3. cmake .. -DCMAKE_BUILD_TYPE=Release
#   4. cmake --build .
#
# The output is: build/Contrapunk.component

cmake_minimum_required(VERSION 3.21)

project(contrapunk-au-wrapper
    VERSION 1.0.0
    LANGUAGES C CXX
)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

if (NOT APPLE)
    message(FATAL_ERROR "AU (Audio Unit) is macOS only")
endif()

enable_language(OBJC)
enable_language(OBJCXX)

# Universal binary for Apple Silicon + Intel
set(CMAKE_OSX_ARCHITECTURES "arm64;x86_64" CACHE STRING "macOS architectures")
set(CMAKE_OSX_DEPLOYMENT_TARGET "10.13" CACHE STRING "Minimum macOS version")

# ── Fetch clap-wrapper from our fork ──────────────────────────────────
include(FetchContent)

FetchContent_Declare(
    clap-wrapper
    GIT_REPOSITORY https://github.com/contrapunk-audio/clap-wrapper.git
    GIT_TAG main
)

# Tell clap-wrapper to download CLAP SDK and AudioUnitSDK automatically
set(CLAP_WRAPPER_DOWNLOAD_DEPENDENCIES TRUE CACHE BOOL "" FORCE)
set(CLAP_WRAPPER_BUILD_AUV2 ON CACHE BOOL "" FORCE)

FetchContent_MakeAvailable(clap-wrapper)

FetchContent_GetProperties(clap-wrapper SOURCE_DIR CLAP_WRAPPER_SOURCE_DIR)

# Logic may request the AU Cocoa view before the returned NSView has an
# NSWindow. nih-plug-webview/wry needs that window during CLAP set_parent(),
# so defer CLAP GUI creation/parenting until viewDidMoveToWindow.
set(_defer_webview_parent_patch "${CMAKE_CURRENT_SOURCE_DIR}/patches/defer-auv2-webview-parent.patch")
execute_process(
    COMMAND git apply --check "${_defer_webview_parent_patch}"
    WORKING_DIRECTORY "${CLAP_WRAPPER_SOURCE_DIR}"
    RESULT_VARIABLE _defer_webview_parent_patch_check
    OUTPUT_QUIET
    ERROR_QUIET
)
if (_defer_webview_parent_patch_check EQUAL 0)
    execute_process(
        COMMAND git apply "${_defer_webview_parent_patch}"
        WORKING_DIRECTORY "${CLAP_WRAPPER_SOURCE_DIR}"
        COMMAND_ERROR_IS_FATAL ANY
    )
else()
    file(READ "${CLAP_WRAPPER_SOURCE_DIR}/src/detail/auv2/wrappedview.asinclude.mm" _wrappedview_src)
    string(FIND "${_wrappedview_src}" "attachCLAPGui" _defer_webview_parent_already_applied)
    if (_defer_webview_parent_already_applied EQUAL -1)
        message(FATAL_ERROR "Failed to apply clap-wrapper AUv2 WebView parenting patch")
    endif()
endif()

# Logic accepts the AU's too-small requested editor size, cropping the Svelte UI.
set(_clamp_editor_size_patch "${CMAKE_CURRENT_SOURCE_DIR}/patches/clamp-auv2-editor-size.patch")
execute_process(
    COMMAND git apply --check "${_clamp_editor_size_patch}"
    WORKING_DIRECTORY "${CLAP_WRAPPER_SOURCE_DIR}"
    RESULT_VARIABLE _clamp_editor_size_patch_check
    OUTPUT_QUIET
    ERROR_QUIET
)
if (_clamp_editor_size_patch_check EQUAL 0)
    execute_process(
        COMMAND git apply "${_clamp_editor_size_patch}"
        WORKING_DIRECTORY "${CLAP_WRAPPER_SOURCE_DIR}"
        COMMAND_ERROR_IS_FATAL ANY
    )
else()
    file(READ "${CLAP_WRAPPER_SOURCE_DIR}/src/detail/auv2/wrappedview.asinclude.mm" _wrappedview_src)
    file(READ "${CLAP_WRAPPER_SOURCE_DIR}/src/detail/auv2/auv2_shared.mm" _auv2_shared_src)
    string(FIND "${_wrappedview_src}\n${_auv2_shared_src}" "setContentMinSize" _clamp_editor_size_already_applied)
    if (_clamp_editor_size_already_applied EQUAL -1)
        message(FATAL_ERROR "Failed to apply clap-wrapper AUv2 editor size patch")
    endif()
endif()

# ── Path to pre-built CLAP bundle ────────────────────────────────────
set(CLAP_BUNDLE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../target/bundled/Contrapunk.clap"
    CACHE PATH "Path to pre-built .clap bundle")

if (NOT EXISTS "${CLAP_BUNDLE_PATH}")
    message(FATAL_ERROR
        "CLAP bundle not found at ${CLAP_BUNDLE_PATH}\n"
        "Build it first: cargo xtask bundle contrapunk_plugin --release --features embed-ui")
endif()

# ── AUv2 wrapper target ──────────────────────────────────────────────
set(CONTRAPUNK_AUV2_OUTPUT_NAME "Contrapunk" CACHE STRING "AUv2 bundle/display name")
set(CONTRAPUNK_AUV2_BUNDLE_IDENTIFIER "com.contrapunk.harmony.auv2" CACHE STRING "AUv2 bundle identifier")
set(CONTRAPUNK_AUV2_SUBTYPE_CODE "CpHm" CACHE STRING "AUv2 subtype code")
set(CONTRAPUNK_AUV2_MANUFACTURER_CODE "CpAu" CACHE STRING "AUv2 manufacturer code")
set(CONTRAPUNK_AUV2_MANUFACTURER_NAME "Contrapunk Audio" CACHE STRING "AUv2 manufacturer name")

add_library(contrapunk_auv2 MODULE)

target_add_auv2_wrapper(
    TARGET contrapunk_auv2
    OUTPUT_NAME "${CONTRAPUNK_AUV2_OUTPUT_NAME}"

    BUNDLE_IDENTIFIER "${CONTRAPUNK_AUV2_BUNDLE_IDENTIFIER}"
    BUNDLE_VERSION "${PROJECT_VERSION}"

    # AU identification codes (4 chars each)
    # Type: aumi = "audio unit MIDI processor" (note effect)
    INSTRUMENT_TYPE "aumi"
    # Subtype: identifies this specific plugin
    SUBTYPE_CODE "${CONTRAPUNK_AUV2_SUBTYPE_CODE}"
    # Manufacturer: identifies Contrapunk Audio
    MANUFACTURER_CODE "${CONTRAPUNK_AUV2_MANUFACTURER_CODE}"
    MANUFACTURER_NAME "${CONTRAPUNK_AUV2_MANUFACTURER_NAME}"

    # Embed the pre-built CLAP inside the .component bundle
    MACOS_EMBEDDED_CLAP_LOCATION "${CLAP_BUNDLE_PATH}"
)
