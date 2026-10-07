; Odomouse: Windows installer (Inno Setup 6).
; build.ps1 compiles it after the app is built into build\app:
;   ISCC.exe /DAppVersion=1.0.0 windows\installer.iss
; Result: windows\dist\Odomouse-Setup.exe
;
; Installs per user, so no administrator password is asked:
;   %LOCALAPPDATA%\Programs\Odomouse\
; plus a Start menu entry and an uninstaller in "Apps & features".
; Statistics in %APPDATA%\Odomouse\ stay after uninstalling.

#ifndef AppVersion
  #define AppVersion "1.0.0"
#endif

[Setup]
AppId={{7C3F1B2E-5A64-4E0B-9C51-8D2E6A4F9B10}
AppName=Odomouse
AppVersion={#AppVersion}
AppVerName=Odomouse {#AppVersion}
AppPublisher=Odomouse
DefaultDirName={autopf}\Odomouse
DefaultGroupName=Odomouse
DisableProgramGroupPage=yes
DisableDirPage=yes
PrivilegesRequired=lowest
MinVersion=6.1sp1
OutputDir=dist
OutputBaseFilename=Odomouse-Setup
SetupIconFile=Odomouse\Assets\app.ico
UninstallDisplayIcon={app}\Odomouse.exe
UninstallDisplayName=Odomouse
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
; the app was called Mishka Tracker before 1.0 (same AppId): install into the new folder
UsePreviousAppDir=no
UsePreviousGroup=no
; a running copy is stopped in PrepareToInstall, so no "close the app" dialog
CloseApplications=no

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"

[CustomMessages]
en.Launch=Start Odomouse
ru.Launch=Запустить Odomouse
en.Desktop=Create a desktop shortcut
ru.Desktop=Создать ярлык на рабочем столе

[Tasks]
Name: "desktopicon"; Description: "{cm:Desktop}"; Flags: unchecked

[Files]
Source: "build\app\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Odomouse"; Filename: "{app}\Odomouse.exe"
Name: "{autodesktop}\Odomouse"; Filename: "{app}\Odomouse.exe"; Tasks: desktopicon

[Registry]
; "Launch at login" is a switch inside the app; only clean it up here.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; ValueName: "Odomouse"; Flags: uninsdeletevalue
; the old name's autostart entry
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; ValueName: "Mishka Tracker"; Flags: deletevalue

[Run]
Filename: "{app}\Odomouse.exe"; Description: "{cm:Launch}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; save today's numbers and exit; taskkill only if it is still there
Filename: "{app}\Odomouse.exe"; Parameters: "--quit"; Flags: runhidden waituntilterminated; RunOnceId: "QuitApp"
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM Odomouse.exe"; Flags: runhidden; RunOnceId: "StopApp"

[InstallDelete]
; the app under its old name (statistics are in %APPDATA% and are copied over by the app)
Type: filesandordirs; Name: "{userpf}\Mishka Tracker"
Type: files; Name: "{userprograms}\Mishka Tracker.lnk"
Type: files; Name: "{userdesktop}\Mishka Tracker.lnk"

[UninstallDelete]
; WebView2 cache created next to the app by older builds, if any
Type: filesandordirs; Name: "{app}\Odomouse.exe.WebView2"

[Code]
// An update over a running copy: ask it to save and exit ("--quit"),
// then make sure it is gone so its files can be replaced.
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Code: Integer;
  Exe: String;
begin
  Exe := ExpandConstant('{app}\Odomouse.exe');
  if FileExists(Exe) then
    Exec(Exe, '--quit', '', SW_HIDE, ewWaitUntilTerminated, Code);
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM Odomouse.exe /IM MishkaTracker.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  Sleep(400);
  Result := '';
end;
