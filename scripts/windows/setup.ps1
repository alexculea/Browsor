## If you are trying to install, right click and 
## use Run with PowerShell

Add-Type -AssemblyName PresentationCore, PresentationFramework

function RenderWinregTemplate {
  param (
    $AppName,
    $InstallPath,
    $ExePath,
    $Description,
    $Install,
    $Author
  )

  $output_file = $env:TEMP + "\" + $AppName + ".reg";
  $install_template = @"
Windows Registry Editor Version 5.00

[HKEY_LOCAL_MACHINE\SOFTWARE\Classes\{{appName}}\Application]
"AppUserModelId"="{{appName}}"
"ApplicationIcon"="{{exePath}},0"
"ApplicationName"="{{appName}}"
"ApplicationDescription"="{{description}}"
"ApplicationCompany"="{{author}}"

[HKEY_LOCAL_MACHINE\SOFTWARE\Classes\{{appName}}\InstallInfo]
"HideIconsCommand"="powershell.exe {{installPath}}\\setup.ps1"
"ShowIconsCommand"="powershell.exe {{installPath}}\\setup.ps1"
"IconsVisible"="1"
"ReinstallCommand"="powershell.exe {{installPath}}\\scripts\\windows\\setup.ps1"

[HKEY_LOCAL_MACHINE\SOFTWARE\Classes\{{appName}}\DefaultIcon]
@="{{exePath}},0"

[HKEY_LOCAL_MACHINE\SOFTWARE\Classes\{{appName}}\shell\open\command]
@="\"{{exePath}}\" \"%1\""


[HKEY_LOCAL_MACHINE\SOFTWARE\{{appName}}\Capabilities]
"ApplicationDescription"="Shows a browser selector"
"ApplicationName"="{{appName}}"
"ApplicationIcon"="{{exePath}},0"

[HKEY_LOCAL_MACHINE\SOFTWARE\{{appName}}\Capabilities\FileAssociations]
".html"="{{appName}}"
".htm"="{{appName}}"
".xht"="{{appName}}"
".xhtml"="{{appName}}"

[HKEY_LOCAL_MACHINE\SOFTWARE\{{appName}}\Capabilities\StartMenu]
"StartMenuInternet"="{{appName}}"

[HKEY_LOCAL_MACHINE\SOFTWARE\{{appName}}\Capabilities\UrlAssociations]
"ftp"="{{appName}}"
"http"="{{appName}}"
"https"="{{appName}}"

[HKEY_LOCAL_MACHINE\SOFTWARE\Clients\StartMenuInternet\{{appName}}\DefaultIcon]
@="{{exePath}},0"

[HKEY_LOCAL_MACHINE\SOFTWARE\Clients\StartMenuInternet\{{appName}}\shell\open\command]
@="\"{{exePath}}\" \"%1\""

[HKEY_LOCAL_MACHINE\SOFTWARE\Clients\StartMenuInternet\{{appName}}\Capabilities]
"ApplicationDescription"="{{description}}"
"ApplicationName"="{{appName}}"
"ApplicationIcon"="{{exePath}},0"

[HKEY_LOCAL_MACHINE\SOFTWARE\Clients\StartMenuInternet\{{appName}}\Capabilities\FileAssociations]
".html"="{{appName}}"
".htm"="{{appName}}"
".xht"="{{appName}}"
".xhtml"="{{appName}}"

[HKEY_LOCAL_MACHINE\SOFTWARE\Clients\StartMenuInternet\{{appName}}\Capabilities\StartMenu]
"StartMenuInternet"="{{appName}}"

[HKEY_LOCAL_MACHINE\SOFTWARE\Clients\StartMenuInternet\{{appName}}\Capabilities\UrlAssociations]
"ftp"="{{appName}}"
"http"="{{appName}}"
"https"="{{appName}}"

[HKEY_LOCAL_MACHINE\SOFTWARE\RegisteredApplications]
"{{appName}}"="SOFTWARE\\Clients\\StartMenuInternet\\{{appName}}\\Capabilities"
"@;

$uninstall_template = @"
Windows Registry Editor Version 5.00

[-HKEY_LOCAL_MACHINE\SOFTWARE\Classes\{{appName}}]
[-HKEY_LOCAL_MACHINE\SOFTWARE\{{appName}}]
[-HKEY_LOCAL_MACHINE\SOFTWARE\Clients\StartMenuInternet\{{appName}}]
[HKEY_LOCAL_MACHINE\SOFTWARE\RegisteredApplications]
"{{appName}}"=-
"@;

  $template = If ($Install) { $install_template } Else { $uninstall_template };
  
  # WinReg requires strings to escape the \
  $InstallPathEscaped = $InstallPath.replace('\', '\\');
  $ExePathEscaped = $ExePath.replace('\', '\\');

  $output = $template.replace('{{appName}}', $AppName);
  $output = $output.replace('{{installPath}}', $InstallPathEscaped);
  $output = $output.replace('{{exePath}}', $ExePathEscaped);
  $output = $output.replace('{{description}}', $Description);
  $output = $output.replace('{{author}}', $Author);

  Set-Content -Path $output_file -Value $output;
  return $output_file;
}

function FindExePath {
  param (
    $AppName
  )

  if ($AppName -Eq "") {
    Write-Host("AppName not given");
    exit;
  }

  $searchPaths = @(
    ($PSScriptRoot + "\" + $AppName + ".exe"),
    $PSScriptRoot + "..\..\target\release\" + $AppName + ".exe",
    $PSScriptRoot + "..\..\target\debug\" + $AppName + ".exe"
  );

  Foreach ($path in $searchPaths) {
    if (Test-Path $path -PathType Leaf) {
      $result = Resolve-Path $path;
      $result = $result.toString();
      Write-Host("Using .exe file: " + $result);
      return $result;
    }
  }

  Write-Host("Could not find .exe file");
  Write-Host("Expecting to find it in " + $searchPaths);
  exit;
}

function InformUserAboutRights {
  if (!
    #current role
      (New-Object Security.Principal.WindowsPrincipal(
      [Security.Principal.WindowsIdentity]::GetCurrent()
      #is admin?
    )).IsInRole(
      [Security.Principal.WindowsBuiltInRole]::Administrator
    )
  ) {
    [System.Windows.MessageBox]::Show(
      'You need to run this as administrator.',
      'Error',
      'Ok',
      'Error'
    );

    exit
  }
}

function GetInstallFiles {
  param (
    $AppName
  )

  $dstDir = GetInstallDir -AppName $AppName;
  $dstExe = GetExeInstallPath -AppName $AppName;
  $dstSetupFile = $dstDir + "\setup.ps1";

  $srcExe = FindExePath -AppName $AppName;
  $srcSetupFile = $PSScriptRoot + "\" + "setup.ps1";

  return @(
    @($srcExe, $dstExe),
    @($srcSetupFile, $dstSetupFile)
  );
}

function CreateInstallDir {
  param (
    $AppName
  )

  $sys_dir = GetInstallDirParent;
  $name = $AppName;
  $path = GetInstallDir;
  if (Test-Path $path -PathType Leaf) {
    return;
  }

  New-Item -Path $sys_dir -Name $name -ItemType "directory";
}

function CopyInstallFiles {
  param (
    $AppNamee
  )

  $files = GetInstallFiles -AppName $AppName;
  CreateInstallDir -AppName $AppName;

  Foreach ($entry in $files) {
    $src = $entry[0];
    $dst = $entry[1];

    if ($src -And $dst) {
      Copy-Item $src -Destination $dst;
    }
  }
}

function DeleteInstalledFiles {
  param (
    $AppName
  )

  $installDir = GetInstallDir -AppName $AppName;
  Remove-Item -Path $installDir -Recurse;
}

function GetInstallDir {
  param (
    $AppName
  )

  $sys_dir = GetInstallDirParent;
  return $sys_dir + "\" + $AppName;
}

function GetInstallDirParent {
   return $env:LOCALAPPDATA;
}

function GetExeInstallPath {
  param (
    $AppName
  )

  $installDir = GetInstallDir -AppName $AppName;
  return $installDir + "\" + $AppName + ".exe"
}

function IsAlreadyInstalled {
  param (
    $AppName
  )

  $path = GetExeInstallPath -AppName $AppName;
  return Test-Path $path -PathType Leaf;
}


$AppName = "Browsor";
$Author = "Alex Culea";
$Description = "A tool that registers as the default system browser prompting you to select one of the installed ones to open the URL. Works whenever you open a link from any desktop app.";
$InstallExePath = GetExeInstallPath -AppName $AppName;
$InstallDir = GetInstallDir -AppName $AppName;

if ($args[0] -eq "--inspect") {
  $Install = IsAlreadyInstalled -AppName $AppName
  $WinRegFile = RenderWinregTemplate `
    -AppName $AppName `
    -InstallPath $InstallDir `
    -ExePath $InstallExePath `
    -Install $Install `
    -Description $Description `
    -Author $Author;
  
  Rename-Item -Path $WinRegFile -NewName ($WinRegFile + ".txt");
  start ($WinRegFile + ".txt");
  exit
}

if (IsAlreadyInstalled -AppName $AppName) {
  $user_result = [System.Windows.MessageBox]::Show(
    "This action will:`n - Remove the program files from the local AppData folder`n - Remove all the associated registry entries`n`nTo inspect the content of the registry, rerun the script with `"setup.ps1 --inspect`"`n`n`nDo you want to uninstall?",
    'Install',
    "YesNo",
    'Info'
  );

  if ($user_result -ne "Yes") {
    exit
  }

  InformUserAboutRights

  $WinRegFile = RenderWinregTemplate `
    -AppName $AppName `
    -InstallPath $InstallDir `
    -ExePath $InstallExePath `
    -Install $false `
    -Description $Description `
    -Author $Author

  reg import $WinRegFile;
  Remove-Item  $WinRegFile;
  DeleteInstalledFiles -AppName $AppName
} else {
  $user_result = [System.Windows.MessageBox]::Show(
    "This action will:`n - Copy the program files to the local AppData folder`n - Add the needed registry to register the program as a browser`n`nTo inspect the content of the registry, rerun the script with `"setup.ps1 --inspect`"`n`n`nDo you want to install?",
    'Install',
    "YesNo",
    'Info'
  );

  if ($user_result -eq 'Yes') {
    InformUserAboutRights

    $SrcExePath = FindExePath -AppName $AppName
    $WinRegFile = RenderWinregTemplate `
      -AppName $AppName `
      -InstallPath "$InstallDir" `
      -ExePath "$InstallExePath" `
      -Install $true `
      -Description $Description `
      -Author $Author

    CopyInstallFiles -AppName $AppName
    reg import $WinRegFile;
    Remove-Item  $WinRegFile;
  }
}
