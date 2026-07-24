Unicode True
!include "MUI2.nsh"

!ifndef OUTFILE
  !error "OUTFILE is required"
!endif
!ifndef CONTRAPUNK_VST3
  !error "CONTRAPUNK_VST3 is required"
!endif
!ifndef CONTRAPUNK_CLAP
  !error "CONTRAPUNK_CLAP is required"
!endif
!ifndef ELIXIR_VST3
  !error "ELIXIR_VST3 is required"
!endif
!ifndef ELIXIR_CLAP
  !error "ELIXIR_CLAP is required"
!endif

Name "Contrapunk Audio Plug-ins"
OutFile "${OUTFILE}"
InstallDir "$PROGRAMFILES64\Contrapunk Audio Plug-ins"
RequestExecutionLevel admin
SetCompressor /SOLID lzma
ShowInstDetails show
ShowUninstDetails show

!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

SectionGroup /e "Contrapunk (required)" SECGRP_CONTRAPUNK
  Section "Contrapunk VST3" SEC_CONTRAPUNK_VST3
    SectionIn RO
    SetOutPath "$COMMONFILES64\VST3\Contrapunk.vst3"
    File /r "${CONTRAPUNK_VST3}\*"
  SectionEnd

  Section "Contrapunk CLAP" SEC_CONTRAPUNK_CLAP
    SectionIn RO
    SetOutPath "$COMMONFILES64\CLAP"
    File "${CONTRAPUNK_CLAP}"
  SectionEnd
SectionGroupEnd

SectionGroup /e "Elixir (optional, selected by default)" SECGRP_ELIXIR
  Section "Elixir VST3" SEC_ELIXIR_VST3
    SetOutPath "$COMMONFILES64\VST3\Elixir.vst3"
    File /r "${ELIXIR_VST3}\*"
  SectionEnd

  Section "Elixir CLAP" SEC_ELIXIR_CLAP
    SetOutPath "$COMMONFILES64\CLAP"
    File "${ELIXIR_CLAP}"
  SectionEnd
SectionGroupEnd

Section -Registry
  SetOutPath "$INSTDIR"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Contrapunk Audio Plug-ins" "DisplayName" "Contrapunk Audio Plug-ins"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Contrapunk Audio Plug-ins" "Publisher" "Contrapunk Audio"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Contrapunk Audio Plug-ins" "UninstallString" '"$INSTDIR\Uninstall.exe"'
SectionEnd

Section "Uninstall"
  RMDir /r "$COMMONFILES64\VST3\Contrapunk.vst3"
  Delete "$COMMONFILES64\CLAP\Contrapunk.clap"
  RMDir /r "$COMMONFILES64\VST3\Elixir.vst3"
  Delete "$COMMONFILES64\CLAP\Elixir.clap"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Contrapunk Audio Plug-ins"
SectionEnd
