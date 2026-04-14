; installer.iss

[Setup]
AppName=RSVP Generator
AppVersion=1.0.2
AppPublisher=Your Name
AppPublisherURL=https://github.com/Horera-dev/rsvp
DefaultDirName={autopf}\RSVPGenerator
DefaultGroupName=RSVP Generator
OutputDir=installer_output
OutputBaseFilename=RSVPGenerator-Setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern

; Minimum Windows version: Windows 10
MinVersion=10.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french";  MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"

[Files]
; Main binary
Source: "release\rsvp-generator.exe"; DestDir: "{app}"; Flags: ignoreversion

; ffmpeg
Source: "release\ffmpeg.exe"; DestDir: "{app}"; Flags: ignoreversion

; Config and assets
Source: "release\configuration.toml"; DestDir: "{app}"; Flags: ignoreversion
Source: "release\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs

; Create empty out/ directory
Source: "release\out\.gitkeep"; DestDir: "{app}\out"; Flags: ignoreversion

[Icons]
; Start menu shortcut
Name: "{group}\RSVP Generator"; Filename: "{app}\rsvp-generator.exe"
Name: "{group}\Uninstall RSVP Generator"; Filename: "{uninstallexe}"

; Desktop shortcut (optional, only if user chose it)
Name: "{autodesktop}\RSVP Generator"; Filename: "{app}\rsvp-generator.exe"; Tasks: desktopicon

[Run]
; Offer to open the install folder after installation
Filename: "{app}"; Description: "Open installation folder"; Flags: postinstall shellexec skipifsilent

[Code]
// Optional: check if Visual C++ redistributable is needed
// Most Windows 10+ machines have it, so usually not necessary for Rust binaries