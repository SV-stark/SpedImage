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

!define APP_NAME "SpedImage"
!define PROG_ID "SpedImage.AssocFile"

!macro RegisterExtension ext
    WriteRegStr HKLM "Software\Classes\.${ext}" "" "${PROG_ID}"
    WriteRegStr HKLM "Software\Classes\.${ext}\OpenWithProgids" "${PROG_ID}" ""
    WriteRegStr HKLM "Software\${APP_NAME}\Capabilities\FileAssociations" ".${ext}" "${PROG_ID}"
!macroend

!macro UnregisterExtension ext
    DeleteRegValue HKLM "Software\Classes\.${ext}\OpenWithProgids" "${PROG_ID}"
    
    ; Only delete the default value if it points to us
    ReadRegStr $0 HKLM "Software\Classes\.${ext}" ""
    StrCmp $0 "${PROG_ID}" 0 +2
        DeleteRegValue HKLM "Software\Classes\.${ext}" ""
!macroend

Section "Install"
    SetOutPath "$INSTDIR"
    File "target\release\spedimage.exe"
    File "assets\icons\icon.ico"
    
    WriteUninstaller "$INSTDIR\uninstall.exe"
    
    CreateShortCut "$DESKTOP\SpedImage.lnk" "$INSTDIR\spedimage.exe" "" "$INSTDIR\icon.ico"
    
    CreateDirectory "$SMPROGRAMS\SpedImage"
    CreateShortCut "$SMPROGRAMS\SpedImage\SpedImage.lnk" "$INSTDIR\spedimage.exe" "" "$INSTDIR\icon.ico"
    CreateShortCut "$SMPROGRAMS\SpedImage\Uninstall.lnk" "$INSTDIR\uninstall.exe"
    
    ; Register file associations (Modern Windows 10/11 approach)
    SetRegView 64
    
    ; 1. Create the ProgID (The actual handler)
    WriteRegStr HKLM "Software\Classes\${PROG_ID}" "" "SpedImage Image File"
    WriteRegStr HKLM "Software\Classes\${PROG_ID}\DefaultIcon" "" "$INSTDIR\spedimage.exe,0"
    WriteRegStr HKLM "Software\Classes\${PROG_ID}\shell\open\command" "" '"$INSTDIR\spedimage.exe" "%1"'
    
    ; 2. Define Application Capabilities
    WriteRegStr HKLM "Software\${APP_NAME}\Capabilities" "ApplicationName" "${APP_NAME}"
    WriteRegStr HKLM "Software\${APP_NAME}\Capabilities" "ApplicationDescription" "Ultra-Lightweight GPU-Accelerated Image Viewer"
    WriteRegStr HKLM "Software\${APP_NAME}\Capabilities" "ApplicationIcon" "$INSTDIR\icon.ico,0"
    
    ; 3. Register individual extensions
    !insertmacro RegisterExtension "jpg"
    !insertmacro RegisterExtension "jpeg"
    !insertmacro RegisterExtension "png"
    !insertmacro RegisterExtension "gif"
    !insertmacro RegisterExtension "bmp"
    !insertmacro RegisterExtension "tga"
    !insertmacro RegisterExtension "tiff"
    !insertmacro RegisterExtension "tif"
    !insertmacro RegisterExtension "webp"
    !insertmacro RegisterExtension "ico"
    !insertmacro RegisterExtension "avif"
    !insertmacro RegisterExtension "svg"
    
    ; 4. Register the application in the global RegisteredApplications list
    WriteRegStr HKLM "Software\RegisteredApplications" "${APP_NAME}" "Software\${APP_NAME}\Capabilities"
    
    ; 5. Notify Windows that associations have changed
    System::Call 'Shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)' ; SHCNE_ASSOCCHANGED
SectionEnd

Section "Uninstall"
    SetRegView 64
    
    Delete "$DESKTOP\SpedImage.lnk"
    RMDir /r "$SMPROGRAMS\SpedImage"
    
    Delete "$INSTDIR\spedimage.exe"
    Delete "$INSTDIR\icon.ico"
    Delete "$INSTDIR\uninstall.exe"
    
    ; Remove registry file associations
    DeleteRegKey HKLM "Software\Classes\${PROG_ID}"
    
    !insertmacro UnregisterExtension "jpg"
    !insertmacro UnregisterExtension "jpeg"
    !insertmacro UnregisterExtension "png"
    !insertmacro UnregisterExtension "gif"
    !insertmacro UnregisterExtension "bmp"
    !insertmacro UnregisterExtension "tga"
    !insertmacro UnregisterExtension "tiff"
    !insertmacro UnregisterExtension "tif"
    !insertmacro UnregisterExtension "webp"
    !insertmacro UnregisterExtension "ico"
    !insertmacro UnregisterExtension "avif"
    !insertmacro UnregisterExtension "svg"
    
    DeleteRegKey HKLM "Software\${APP_NAME}"
    DeleteRegValue HKLM "Software\RegisteredApplications" "${APP_NAME}"

    RMDir "$INSTDIR"
    
    ; Refresh shell
    System::Call 'Shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
SectionEnd
