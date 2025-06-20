[Setup]
AppName=Templatize
AppVersion={#SetupSetting("AppVersion")}
AppPublisher=Archetect
AppPublisherURL=https://archetect.github.io
AppSupportURL=https://github.com/archetect/templatize
AppUpdatesURL=https://github.com/archetect/templatize/releases
DefaultDirName={autopf}\Templatize
DefaultGroupName=Templatize
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\target\installer
OutputBaseFilename=templatize-{#SetupSetting("AppVersion")}-windows-x86_64-installer
; SetupIconFile=icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ChangesEnvironment=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "..\..\target\release\templatize.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Templatize"; Filename: "{app}\templatize.exe"
Name: "{group}\{cm:UninstallProgram,Templatize}"; Filename: "{uninstallexe}"

[Tasks]
Name: "addtopath"; Description: "Add to PATH"; GroupDescription: "Additional options:"

[Code]
procedure CurStepChanged(CurStep: TSetupStep);
var
  AppPath: string;
  Paths: string;
begin
  if (CurStep = ssPostInstall) and IsTaskSelected('addtopath') then
  begin
    AppPath := ExpandConstant('{app}');
    if RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Paths) then
    begin
      if Pos(UpperCase(AppPath), UpperCase(Paths)) = 0 then
      begin
        Paths := Paths + ';' + AppPath;
        RegWriteStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Paths);
      end;
    end
    else
    begin
      RegWriteStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', AppPath);
    end;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  AppPath: string;
  Paths: string;
  P: Integer;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    AppPath := ExpandConstant('{app}');
    if RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Paths) then
    begin
      P := Pos(UpperCase(';' + AppPath), UpperCase(Paths));
      if P = 0 then
        P := Pos(UpperCase(AppPath + ';'), UpperCase(Paths));
      if P = 0 then
        P := Pos(UpperCase(AppPath), UpperCase(Paths));
      if P > 0 then
      begin
        Delete(Paths, P, Length(AppPath) + 1);
        if (Length(Paths) > 0) and (Paths[Length(Paths)] = ';') then
          Delete(Paths, Length(Paths), 1);
        RegWriteStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Paths);
      end;
    end;
  end;
end;