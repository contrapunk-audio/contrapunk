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

# Logic 11/macOS 26 crashes AUHostingService when clap-wrapper advertises a
# CLAP webview through AUv2 CocoaUI. The AU uses Logic's generic parameter UI;
# VST3/CLAP still keep the embedded Svelte UI.
set(_wrapasauv2 "${CLAP_WRAPPER_SOURCE_DIR}/src/wrapasauv2.cpp")
file(READ "${_wrapasauv2}" _wrapasauv2_src)
set(_wrapasauv2_before "${_wrapasauv2_src}")
string(REPLACE [[      case kAudioUnitProperty_CocoaUI:
        if (!_plugin->_ext._gui) return kAudioUnitErr_InvalidProperty;
        if (!_plugin->_ext._gui->is_api_supported(_plugin->_plugin, CLAP_WINDOW_API_COCOA, false))
          return kAudioUnitErr_InvalidProperty;
        outWritable = false;
        outDataSize = sizeof(struct AudioUnitCocoaViewInfo);
        return noErr;
        break;
]] [[      case kAudioUnitProperty_CocoaUI:
        return kAudioUnitErr_InvalidProperty;
]] _wrapasauv2_src "${_wrapasauv2_src}")
string(REPLACE [[      case kAudioUnitProperty_CocoaUI:
        LOGINFO("[clap-wrapper] Property: kAudioUnitProperty_CocoaUI {}",
                (_plugin) ? "plugin" : "no plugin");
        if (_plugin && _plugin->_ext._gui &&
            (_plugin->_ext._gui->is_api_supported(_plugin->_plugin, CLAP_WINDOW_API_COCOA, false)))
        {
          fillAudioUnitCocoaView(((AudioUnitCocoaViewInfo *)outData), _plugin);
          LOGINFO("[clap-wrapper] kAudioUnitProperty_CocoaUI complete");
          return noErr;  // sizeof(AudioUnitCocoaViewInfo);
        }
        else
        {
          LOGINFO("[clap-wrapper] Mysterious: kAudioUnitProperty_CocoaUI although now plugin ext");
          fillAudioUnitCocoaView(((AudioUnitCocoaViewInfo *)outData), _plugin);
          return noErr;
        }
        return kAudioUnitErr_InvalidProperty;
        break;
]] [[      case kAudioUnitProperty_CocoaUI:
        return kAudioUnitErr_InvalidProperty;
]] _wrapasauv2_src "${_wrapasauv2_src}")
if (_wrapasauv2_src STREQUAL _wrapasauv2_before)
    string(FIND "${_wrapasauv2_src}" "case kAudioUnitProperty_CocoaUI:\n        return kAudioUnitErr_InvalidProperty;" _cocoa_ui_already_disabled)
    if (_cocoa_ui_already_disabled EQUAL -1)
        message(FATAL_ERROR "Failed to patch clap-wrapper AUv2 Cocoa UI")
    endif()
endif()
file(WRITE "${_wrapasauv2}" "${_wrapasauv2_src}")

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
