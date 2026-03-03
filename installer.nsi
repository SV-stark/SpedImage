!include "MUI2.nsh"

Name "SpedImage"
OutFile "SpedImage_Setup.exe"
InstallDir "$PROGRAMFILES64\SpedImage"
RequestExecutionLevel admin

!define MUI_ICON "assets\icons\icon.png"
!define MUI_UNICON "assets\icons\icon.png"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section "Install"
    SetOutPath "$INSTDIR"
    File "target\release\spedimage.exe"
    File "assets\icons\icon.png"
    
    WriteUninstaller "$INSTDIR\uninstall.exe"
    
    CreateShortCut "$DESKTOP\SpedImage.lnk" "$INSTDIR\spedimage.exe" "" "$INSTDIR\icon.png"
    
    CreateDirectory "$SMPROGRAMS\SpedImage"
    CreateShortCut "$SMPROGRAMS\SpedImage\SpedImage.lnk" "$INSTDIR\spedimage.exe" "" "$INSTDIR\icon.png"
    CreateShortCut "$SMPROGRAMS\SpedImage\Uninstall.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$DESKTOP\SpedImage.lnk"
    RMDir /r "$SMPROGRAMS\SpedImage"
    
    Delete "$INSTDIR\spedimage.exe"
    Delete "$INSTDIR\icon.png"
    Delete "$INSTDIR\uninstall.exe"
    
    RMDir "$INSTDIR"
SectionEnd
