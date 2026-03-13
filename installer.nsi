!include "MUI2.nsh"

Name "SpedImage"
OutFile "SpedImage_Setup.exe"
InstallDir "$PROGRAMFILES64\SpedImage"
RequestExecutionLevel admin

!define MUI_ICON "assets\icons\icon.ico"
!define MUI_UNICON "assets\icons\icon.ico"

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
    File "target/release/spedimage.exe"
    File "assets/icons/icon.ico"
    File /r "assets/libheif/*.dll"
    
    WriteUninstaller "$INSTDIR\uninstall.exe"
    
    CreateShortCut "$DESKTOP\SpedImage.lnk" "$INSTDIR\spedimage.exe" "" "$INSTDIR\icon.ico"
    
    CreateDirectory "$SMPROGRAMS\SpedImage"
    CreateShortCut "$SMPROGRAMS\SpedImage\SpedImage.lnk" "$INSTDIR\spedimage.exe" "" "$INSTDIR\icon.ico"
    CreateShortCut "$SMPROGRAMS\SpedImage\Uninstall.lnk" "$INSTDIR\uninstall.exe"
    
    ; Register file associations
    WriteRegStr HKCU "Software\Classes\SpedImage.Image" "" "SpedImage Image File"
    WriteRegStr HKCU "Software\Classes\SpedImage.Image\DefaultIcon" "" "$INSTDIR\spedimage.exe,0"
    WriteRegStr HKCU "Software\Classes\SpedImage.Image\shell\open\command" "" '"$INSTDIR\spedimage.exe" "%1"'
SectionEnd

Section "Uninstall"
    Delete "$DESKTOP\SpedImage.lnk"
    RMDir /r "$SMPROGRAMS\SpedImage"
    
    Delete "$INSTDIR\spedimage.exe"
    Delete "$INSTDIR\icon.ico"
    Delete "$INSTDIR\uninstall.exe"
    
    ; Remove registry file associations
    DeleteRegKey HKCU "Software\Classes\SpedImage.Image"

    RMDir "$INSTDIR"
SectionEnd
