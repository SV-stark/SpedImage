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
    WriteRegStr HKLM "Software\Classes\.${ext}" "PerceivedType" "image"
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
    WriteRegStr HKLM "Software\Classes\${PROG_ID}" "PerceivedType" "image"
    WriteRegStr HKLM "Software\Classes\${PROG_ID}\DefaultIcon" "" "$INSTDIR\icon.ico"
    WriteRegStr HKLM "Software\Classes\${PROG_ID}\shell\open\command" "" '"$INSTDIR\spedimage.exe" "%1"'
    ; Register Windows thumbnail provider so photo previews are displayed in Windows Explorer instead of a blank white sheet:
    WriteRegStr HKLM "Software\Classes\${PROG_ID}\ShellEx\{e357fccd-a995-4576-b01f-234630154e96}" "" "{C7657C4A-9F68-40fa-A4DF-96BC08EB3551}"
    
    ; 2. Define Application Capabilities
    WriteRegStr HKLM "Software\${APP_NAME}\Capabilities" "ApplicationName" "${APP_NAME}"
    WriteRegStr HKLM "Software\${APP_NAME}\Capabilities" "ApplicationDescription" "Ultra-Lightweight GPU-Accelerated Image Viewer"
    WriteRegStr HKLM "Software\${APP_NAME}\Capabilities" "ApplicationIcon" "$INSTDIR\icon.ico"
    
    ; 3. Register individual extensions
    !insertmacro RegisterExtension "jpg"
    !insertmacro RegisterExtension "jpeg"
    !insertmacro RegisterExtension "png"
    !insertmacro RegisterExtension "gif"
    !insertmacro RegisterExtension "bmp"
    !insertmacro RegisterExtension "tiff"
    !insertmacro RegisterExtension "tif"
    !insertmacro RegisterExtension "webp"
    !insertmacro RegisterExtension "ico"
    !insertmacro RegisterExtension "cur"
    ; No .avif: AVIF is an HEIF container holding AV1, and there is no AV1
    ; decoder, so the app would reject it. Registering it would send users to
    ; an error dialog from what looks like a supported file type.
    !insertmacro RegisterExtension "svg"
    !insertmacro RegisterExtension "heic"
    !insertmacro RegisterExtension "heif"
    !insertmacro RegisterExtension "jxl"
    !insertmacro RegisterExtension "qoi"
    !insertmacro RegisterExtension "exr"
    !insertmacro RegisterExtension "psd"
    !insertmacro RegisterExtension "psb"
    !insertmacro RegisterExtension "hdr"
    !insertmacro RegisterExtension "ppm"
    !insertmacro RegisterExtension "pgm"
    !insertmacro RegisterExtension "pbm"
    !insertmacro RegisterExtension "pnm"
    !insertmacro RegisterExtension "pam"
    !insertmacro RegisterExtension "ff"
    !insertmacro RegisterExtension "arw"
    !insertmacro RegisterExtension "cr2"
    !insertmacro RegisterExtension "crw"
    !insertmacro RegisterExtension "nef"
    !insertmacro RegisterExtension "nrw"
    !insertmacro RegisterExtension "dng"
    !insertmacro RegisterExtension "orf"
    !insertmacro RegisterExtension "raf"
    !insertmacro RegisterExtension "sr2"
    !insertmacro RegisterExtension "srf"
    !insertmacro RegisterExtension "srw"
    !insertmacro RegisterExtension "pef"
    !insertmacro RegisterExtension "mrw"
    !insertmacro RegisterExtension "kdc"
    !insertmacro RegisterExtension "dcr"
    !insertmacro RegisterExtension "rw2"
    ; No .tga: there is no Targa decoder, so the app cannot open one.
    
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
    !insertmacro UnregisterExtension "tiff"
    !insertmacro UnregisterExtension "tif"
    !insertmacro UnregisterExtension "webp"
    !insertmacro UnregisterExtension "ico"
    !insertmacro UnregisterExtension "cur"
    ; These three are unregistered even though they are no longer registered:
    ; people upgrading from an older build already have the association on
    ; disk, and this is the only chance to take it back off.
    ; .avif needs an AV1 decoder that does not exist; .tga has no decoder.
    !insertmacro UnregisterExtension "avif"
    !insertmacro UnregisterExtension "tga"
    !insertmacro UnregisterExtension "svg"
    !insertmacro UnregisterExtension "heic"
    !insertmacro UnregisterExtension "heif"
    !insertmacro UnregisterExtension "jxl"
    !insertmacro UnregisterExtension "qoi"
    !insertmacro UnregisterExtension "exr"
    !insertmacro UnregisterExtension "psd"
    !insertmacro UnregisterExtension "psb"
    !insertmacro UnregisterExtension "hdr"
    !insertmacro UnregisterExtension "ppm"
    !insertmacro UnregisterExtension "pgm"
    !insertmacro UnregisterExtension "pbm"
    !insertmacro UnregisterExtension "pnm"
    !insertmacro UnregisterExtension "pam"
    !insertmacro UnregisterExtension "ff"
    !insertmacro UnregisterExtension "arw"
    !insertmacro UnregisterExtension "cr2"
    !insertmacro UnregisterExtension "crw"
    !insertmacro UnregisterExtension "nef"
    !insertmacro UnregisterExtension "nrw"
    !insertmacro UnregisterExtension "dng"
    !insertmacro UnregisterExtension "orf"
    !insertmacro UnregisterExtension "raf"
    !insertmacro UnregisterExtension "sr2"
    !insertmacro UnregisterExtension "srf"
    !insertmacro UnregisterExtension "srw"
    !insertmacro UnregisterExtension "pef"
    !insertmacro UnregisterExtension "mrw"
    !insertmacro UnregisterExtension "kdc"
    !insertmacro UnregisterExtension "dcr"
    !insertmacro UnregisterExtension "rw2"
    
    DeleteRegKey HKLM "Software\${APP_NAME}"
    DeleteRegValue HKLM "Software\RegisteredApplications" "${APP_NAME}"

    RMDir "$INSTDIR"
    
    ; Refresh shell
    System::Call 'Shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
SectionEnd
