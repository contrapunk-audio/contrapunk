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

# ── Path to pre-built CLAP bundle ────────────────────────────────────
set(CLAP_BUNDLE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../target/bundled/Contrapunk.clap"
    CACHE PATH "Path to pre-built .clap bundle")

if (NOT EXISTS "${CLAP_BUNDLE_PATH}")
    message(FATAL_ERROR
        "CLAP bundle not found at ${CLAP_BUNDLE_PATH}\n"
        "Build it first: cargo xtask bundle contrapunk_plugin --release --features embed-ui")
endif()

# ── AUv2 wrapper target ──────────────────────────────────────────────
add_library(contrapunk_auv2 MODULE)

target_add_auv2_wrapper(
    TARGET contrapunk_auv2
    OUTPUT_NAME "Contrapunk"

    BUNDLE_IDENTIFIER "com.contrapunk.harmony.auv2"
    BUNDLE_VERSION "${PROJECT_VERSION}"

    # AU identification codes (4 chars each)
    # Type: aumi = "audio unit MIDI processor" (note effect)
    INSTRUMENT_TYPE "aumi"
    # Subtype: identifies this specific plugin
    SUBTYPE_CODE "CpHm"
    # Manufacturer: identifies Contrapunk Audio
    MANUFACTURER_CODE "CpAu"
    MANUFACTURER_NAME "Contrapunk Audio"

    # Embed the pre-built CLAP inside the .component bundle
    MACOS_EMBEDDED_CLAP_LOCATION "${CLAP_BUNDLE_PATH}"
)
