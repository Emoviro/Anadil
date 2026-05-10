; Anadil V0.1 Windows installer (NSIS).
;
; Bu script per-user kurulum yapar; admin haklarina gerek yoktur.
; Varsayilan kurulum yolu: %LOCALAPPDATA%\Programs\Anadil
;
; Calistirma (yerel):
;   makensis installer.nsi
;
; Bu script package.ps1 tarafindan target\dist\<dist_name>\ klasoru
; uretildikten sonra ayni klasorden calistirilmasini bekler. Tum
; dosyalar relative path ile ($EXEDIR yerine ../ yapisini bilmiyor)
; dist klasorunden okunur.

Unicode True

; ---------------------------------------------------------------------------
; Surum bilgisi
; ---------------------------------------------------------------------------

!ifndef ANADIL_VERSION
    !define ANADIL_VERSION "0.1.2"
!endif

!ifndef ANADIL_DIST_DIR
    !define ANADIL_DIST_DIR "target\dist\Anadil-v${ANADIL_VERSION}-windows-x64"
!endif

!ifndef ANADIL_OUTFILE
    !define ANADIL_OUTFILE "target\dist\Anadil-Setup-v${ANADIL_VERSION}.exe"
!endif

!define ANADIL_NAME       "Anadil"
!define ANADIL_PUBLISHER  "Emir Canbaz ve Akif Bugra Karsli"
!define ANADIL_URL        "https://github.com/ArsenAlighieri/Anadil"
!define ANADIL_REGKEY     "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anadil"

; ---------------------------------------------------------------------------
; Kurulum genel ayarlari
; ---------------------------------------------------------------------------

Name        "${ANADIL_NAME} v${ANADIL_VERSION}"
OutFile     "${ANADIL_OUTFILE}"
InstallDir  "$LOCALAPPDATA\Programs\Anadil"
InstallDirRegKey HKCU "Software\Anadil" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
ShowInstDetails show
ShowUninstDetails show

VIProductVersion "0.1.2.0"
VIAddVersionKey  "ProductName"     "${ANADIL_NAME}"
VIAddVersionKey  "ProductVersion"  "${ANADIL_VERSION}"
VIAddVersionKey  "FileDescription" "Anadil Setup"
VIAddVersionKey  "FileVersion"     "${ANADIL_VERSION}"
VIAddVersionKey  "CompanyName"     "${ANADIL_PUBLISHER}"
VIAddVersionKey  "LegalCopyright"  "(c) 2026 ${ANADIL_PUBLISHER}"

; ---------------------------------------------------------------------------
; Modern UI 2
; ---------------------------------------------------------------------------

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"

!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN          "$INSTDIR\anadil-ide.exe"
!define MUI_FINISHPAGE_RUN_TEXT     "Anadil IDE'yi simdi calistir"
!define MUI_FINISHPAGE_LINK         "GitHub: ArsenAlighieri/Anadil"
!define MUI_FINISHPAGE_LINK_LOCATION "${ANADIL_URL}"
!define MUI_FINISHPAGE_SHOWREADME   "$INSTDIR\KURULUM.txt"
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Kurulum talimatlarini ac (KURULUM.txt)"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE       "${ANADIL_DIST_DIR}\LICENSE.txt"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "Turkish"
!insertmacro MUI_LANGUAGE "English"

; ---------------------------------------------------------------------------
; Bilesen aciklamalari
; ---------------------------------------------------------------------------

LangString DESC_SecCore     ${LANG_TURKISH} "Anadil derleyici, IDE, runtime ve ornekler. Kaldirilamaz."
LangString DESC_SecCore     ${LANG_ENGLISH} "Anadil compiler, IDE, runtime and examples. Required."
LangString DESC_SecMenu     ${LANG_TURKISH} "Baslat menusunde Anadil grubu olustur."
LangString DESC_SecMenu     ${LANG_ENGLISH} "Add Anadil shortcuts to the Start menu."
LangString DESC_SecPath     ${LANG_TURKISH} "Kullanici PATH'ine Anadil klasorunu ekle (yeni komut isteminde 'anadil' yazabilirsiniz)."
LangString DESC_SecPath     ${LANG_ENGLISH} "Add Anadil to the user PATH so 'anadil' is available from any command prompt."
LangString DESC_SecAssoc    ${LANG_TURKISH} ".ana dosyalarini IDE ile esle (cift tikla -> IDE acilir)."
LangString DESC_SecAssoc    ${LANG_ENGLISH} "Associate .ana files with the IDE (double-click opens the IDE)."

; ---------------------------------------------------------------------------
; Bilesenler
; ---------------------------------------------------------------------------

Section "Anadil cekirdek (zorunlu)" SecCore
    SectionIn RO
    SetOutPath "$INSTDIR"

    ; Dist klasorunden tum dosyalari kopyala
    File "${ANADIL_DIST_DIR}\anadil.exe"
    File "${ANADIL_DIST_DIR}\anadil-ide.exe"
    File "${ANADIL_DIST_DIR}\KURULUM.txt"
    File "${ANADIL_DIST_DIR}\CHANGELOG.txt"
    File "${ANADIL_DIST_DIR}\README.txt"
    File "${ANADIL_DIST_DIR}\LICENSE.txt"

    SetOutPath "$INSTDIR\runtime"
    File "${ANADIL_DIST_DIR}\runtime\anadil_runtime.asm"
    File "${ANADIL_DIST_DIR}\runtime\anadil_runtime.lib"

    SetOutPath "$INSTDIR\examples"
    File "${ANADIL_DIST_DIR}\examples\*.ana"

    SetOutPath "$INSTDIR\docs"
    File "${ANADIL_DIST_DIR}\docs\*.md"

    SetOutPath "$INSTDIR"

    ; Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Kurulum bilgisini Control Panel "Programlari kaldir" listesine yaz
    WriteRegStr HKCU "${ANADIL_REGKEY}" "DisplayName"      "Anadil"
    WriteRegStr HKCU "${ANADIL_REGKEY}" "DisplayVersion"   "${ANADIL_VERSION}"
    WriteRegStr HKCU "${ANADIL_REGKEY}" "Publisher"        "${ANADIL_PUBLISHER}"
    WriteRegStr HKCU "${ANADIL_REGKEY}" "URLInfoAbout"     "${ANADIL_URL}"
    WriteRegStr HKCU "${ANADIL_REGKEY}" "InstallLocation"  "$INSTDIR"
    WriteRegStr HKCU "${ANADIL_REGKEY}" "DisplayIcon"      "$INSTDIR\anadil-ide.exe,0"
    WriteRegStr HKCU "${ANADIL_REGKEY}" "UninstallString"  "$\"$INSTDIR\Uninstall.exe$\""
    WriteRegStr HKCU "${ANADIL_REGKEY}" "QuietUninstallString" "$\"$INSTDIR\Uninstall.exe$\" /S"
    WriteRegDWORD HKCU "${ANADIL_REGKEY}" "NoModify" 1
    WriteRegDWORD HKCU "${ANADIL_REGKEY}" "NoRepair" 1

    ; Kurulum yeri (yeniden kurulumda dogru klasor secilsin)
    WriteRegStr HKCU "Software\Anadil" "InstallDir" "$INSTDIR"
    WriteRegStr HKCU "Software\Anadil" "Version"    "${ANADIL_VERSION}"

    ; Boyut bilgisi
    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD HKCU "${ANADIL_REGKEY}" "EstimatedSize" "$0"
SectionEnd

Section "Baslat menusu kisayollari" SecMenu
    CreateDirectory "$SMPROGRAMS\Anadil"
    CreateShortcut "$SMPROGRAMS\Anadil\Anadil IDE.lnk"        "$INSTDIR\anadil-ide.exe"  "" "$INSTDIR\anadil-ide.exe" 0
    CreateShortcut "$SMPROGRAMS\Anadil\Anadil Komut Istemi.lnk" "$WINDIR\System32\cmd.exe" "/K cd /d $\"$INSTDIR$\""
    CreateShortcut "$SMPROGRAMS\Anadil\Kurulum Bilgisi.lnk"   "$INSTDIR\KURULUM.txt"
    CreateShortcut "$SMPROGRAMS\Anadil\Kaldir.lnk"            "$INSTDIR\Uninstall.exe"
SectionEnd

Section "PATH ortam degiskenine ekle" SecPath
    Push "$INSTDIR"
    Call AddToUserPath
SectionEnd

Section ".ana dosyalarini IDE ile esle" SecAssoc
    WriteRegStr HKCU "Software\Classes\.ana"                                 "" "Anadil.Source"
    WriteRegStr HKCU "Software\Classes\Anadil.Source"                        "" "Anadil Kaynak Dosyasi"
    WriteRegStr HKCU "Software\Classes\Anadil.Source\DefaultIcon"            "" "$INSTDIR\anadil-ide.exe,0"
    WriteRegStr HKCU "Software\Classes\Anadil.Source\shell\open\command"     "" "$\"$INSTDIR\anadil-ide.exe$\" $\"%1$\""
    System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SecCore}  $(DESC_SecCore)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecMenu}  $(DESC_SecMenu)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecPath}  $(DESC_SecPath)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecAssoc} $(DESC_SecAssoc)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; ---------------------------------------------------------------------------
; Build Tools algilamasi (sadece bilgilendirme)
; ---------------------------------------------------------------------------

Function .onInstSuccess
    ; ml64.exe veya link.exe PATH'te var mi diye bakacagiz
    nsExec::ExecToStack 'where /Q link.exe'
    Pop $0
    ${If} $0 != 0
        MessageBox MB_OKCANCEL|MB_ICONINFORMATION \
            "Anadil kuruldu.$\r$\n$\r$\n\
             Native '.exe' derlemesi (anadil derle) icin Visual Studio Build Tools gerekiyor.$\r$\n\
             Su an link.exe PATH'te bulunamadi.$\r$\n$\r$\n\
             Build Tools'u indirme sayfasini acmak icin Tamam'a basin.$\r$\n\
             Sonradan kurmak icin Iptal." \
            IDOK build_tools_open IDCANCEL build_tools_skip
        build_tools_open:
            ExecShell "open" "https://visualstudio.microsoft.com/visual-cpp-build-tools/"
        build_tools_skip:
    ${EndIf}
FunctionEnd

; ---------------------------------------------------------------------------
; PATH yardimcilari (HKCU\Environment)
; ---------------------------------------------------------------------------

; Kullanici PATH'ine bir klasor ekler. Stack uzerinde klasor yolu bekler.
Function AddToUserPath
    Exch $0
    Push $1
    Push $2
    Push $3

    ReadRegStr $1 HKCU "Environment" "Path"

    ; PATH'te zaten var mi? (basit substring kontrol; ayrintili dedupe icin
    ; gerekirse ileride iyilestirilebilir)
    StrLen $2 $0
    StrCpy $3 0
    duplicate_check_loop:
        StrCpy $4 $1 $2 $3
        StrCmp $4 $0 duplicate_found
        StrCmp $4 "" not_duplicate
        IntOp $3 $3 + 1
        Goto duplicate_check_loop

    not_duplicate:
        StrCmp $1 "" path_empty path_append

        path_empty:
            WriteRegExpandStr HKCU "Environment" "Path" "$0"
            Goto broadcast_change

        path_append:
            WriteRegExpandStr HKCU "Environment" "Path" "$1;$0"
            Goto broadcast_change

    duplicate_found:
        ; Zaten var; bir sey yapma
        Goto done

    broadcast_change:
        ; Acik shell oturumlarinin yeni PATH'i farketmesi icin yayin
        SendMessage 0xFFFF 0x001A 0 "STR:Environment" /TIMEOUT=5000

    done:
        Pop $3
        Pop $2
        Pop $1
        Pop $0
FunctionEnd

; ---------------------------------------------------------------------------
; Uninstall
; ---------------------------------------------------------------------------

Section "Uninstall"
    ; Dosyalari sil
    Delete "$INSTDIR\anadil.exe"
    Delete "$INSTDIR\anadil-ide.exe"
    Delete "$INSTDIR\KURULUM.txt"
    Delete "$INSTDIR\CHANGELOG.txt"
    Delete "$INSTDIR\README.txt"
    Delete "$INSTDIR\LICENSE.txt"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir /r "$INSTDIR\runtime"
    RMDir /r "$INSTDIR\examples"
    RMDir /r "$INSTDIR\docs"
    RMDir "$INSTDIR"

    ; Baslat menusu
    Delete "$SMPROGRAMS\Anadil\Anadil IDE.lnk"
    Delete "$SMPROGRAMS\Anadil\Anadil Komut Istemi.lnk"
    Delete "$SMPROGRAMS\Anadil\Kurulum Bilgisi.lnk"
    Delete "$SMPROGRAMS\Anadil\Kaldir.lnk"
    RMDir "$SMPROGRAMS\Anadil"

    ; PATH temizligi
    Push "$INSTDIR"
    Call un.RemoveFromUserPath

    ; Registry temizligi
    DeleteRegKey HKCU "${ANADIL_REGKEY}"
    DeleteRegKey HKCU "Software\Anadil"
    DeleteRegKey HKCU "Software\Classes\.ana"
    DeleteRegKey HKCU "Software\Classes\Anadil.Source"

    ; Shell'e dosya tipi degisikligini bildir
    System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'

    ; Cache klasorunu temizleme bilgisi (silinmez; kullaniciya birakilir)
    ; %LOCALAPPDATA%\Anadil\cache\ varsa kullanici kendi siler.
SectionEnd

Function un.RemoveFromUserPath
    Exch $0
    Push $1
    Push $2
    Push $3
    Push $4

    ReadRegStr $1 HKCU "Environment" "Path"
    ${If} $1 == ""
        Goto done
    ${EndIf}

    ; Uc varyantta arar ve siler:
    ;   ;<INSTDIR>   (sondaki/ortadaki entry)
    ;   <INSTDIR>;   (bastaki entry)
    ;   <INSTDIR>    (tek basina)
    StrCpy $2 ""

    ; ";INSTDIR" kalibi
    Push $1
    Push ";$0"
    Push ""
    Call un.StrReplace
    Pop $1

    ; "INSTDIR;" kalibi
    Push $1
    Push "$0;"
    Push ""
    Call un.StrReplace
    Pop $1

    ; "INSTDIR" kalibi (tek basina)
    Push $1
    Push "$0"
    Push ""
    Call un.StrReplace
    Pop $1

    WriteRegExpandStr HKCU "Environment" "Path" "$1"
    SendMessage 0xFFFF 0x001A 0 "STR:Environment" /TIMEOUT=5000

    done:
    Pop $4
    Pop $3
    Pop $2
    Pop $1
    Pop $0
FunctionEnd

; un. prefix'li StrReplace (uninstaller icin)
Function un.StrReplace
    Exch $R0 ; replace
    Exch
    Exch $R1 ; search
    Exch 2
    Exch $R2 ; haystack
    Push $R3
    Push $R4
    Push $R5
    Push $R6
    StrLen $R3 $R1
    StrCpy $R4 0
    StrCpy $R5 ""
    loop:
        StrCpy $R6 $R2 $R3 $R4
        StrCmp $R6 "" done
        StrCmp $R6 $R1 found
        StrCpy $R6 $R2 1 $R4
        StrCpy $R5 "$R5$R6"
        IntOp $R4 $R4 + 1
        Goto loop
    found:
        StrCpy $R5 "$R5$R0"
        IntOp $R4 $R4 + $R3
        StrCpy $R6 $R2 "" $R4
        StrCpy $R5 "$R5$R6"
        Goto exit
    done:
        ; tail kismi
    exit:
        StrCpy $R0 $R5
        Pop $R6
        Pop $R5
        Pop $R4
        Pop $R3
        Pop $R2
        Pop $R1
        Exch $R0
FunctionEnd
