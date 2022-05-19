Add-Type -AssemblyName PresentationCore, PresentationFramework
function RenderWinregTemplate {
  param (
    $AppName,
    $InstallPath,
    $ExePath,
    $TemplateFile
  )

  $output_file = $env:TEMP + "\browser-selector.reg";
  $template = Get-Content -Path $TemplateFile;
  
  
  # WinReg requires strings to escape the \
  $InstallPathEscaped = $InstallPath.replace('\', '\\');
  $ExePathEscaped = $ExePath.replace('\', '\\');

  $output = $template.replace('{{appName}}', $AppName);
  $output = $output.replace('{{installPath}}', $InstallPathEscaped);
  $output = $output.replace('{{exePath}}', $ExePathEscaped);
  
  Set-Content -Path $output_file -Value $output;
}

function FindExePath {
  $searchPaths = @(
    "browser-selector.exe",
    "..\..\target\release\browser-selector.exe",
    "..\..\target\debug\browser-selector.exe"
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

if ($args[0] -eq "--uninstall") {
  $user_result = [System.Windows.MessageBox]::Show(
    'Do you want to uninstall?',
    'Install',
    "YesNo",
    'Info'
  );

  if ($user_result -ne "Yes") {
    exit
  }

  $ExePath = FindExePath
  InformUserAboutRights

  RenderWinregTemplate `
    -AppName 'browser-selector-1.0' `
    -InstallPath 'C:\Repos\browser-selector-rt' `
    -ExePath $ExePath `
    -TemplateFile '.\remove.reg.tpl';

  reg import $env:TEMP\browser-selector.reg;
  Remove-Item $env:TEMP\browser-selector.reg;
}
else {
  $user_result = [System.Windows.MessageBox]::Show(
    'Do you want to install?',
    'Install',
    "YesNo",
    'Info'
  );

  InformUserAboutRights

  if ($user_result -eq 'Yes') {
    $ExePath = FindExePath
    RenderWinregTemplate `
      -AppName 'browser-selector-1.0' `
      -InstallPath 'C:\Repos\browser-selector-rt' `
      -ExePath $ExePath `
      -TemplateFile '.\install.reg.tpl';

    reg import $env:TEMP\browser-selector.reg;
    Remove-Item $env:TEMP\browser-selector.reg;
  }
}
