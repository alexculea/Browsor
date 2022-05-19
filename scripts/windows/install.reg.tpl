Windows Registry Editor Version 5.00

[HKEY_LOCAL_MACHINE\SOFTWARE\Classes\{{appName}}\Application]
"AppUserModelId"="{{appName}}"
"ApplicationIcon"="{{exePath}},0"
"ApplicationName"="browser-selector-1.0"
"ApplicationDescription"="Access the Internet"
"ApplicationCompany"="Alex Culea"

[HKEY_LOCAL_MACHINE\SOFTWARE\Classes\{{appName}}\InstallInfo]
"HideIconsCommand"="powershell.exe {{installPath}}\\scripts\\windows\\unsupported-command.ps1"
"ShowIconsCommand"="powershell.exe {{installPath}}\\scripts\\windows\\unsupported-command.ps1"
"IconsVisible"="1"
"ReinstallCommand"="{{installPath}}\\scripts\\windows\\install.ps1"

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
"ApplicationDescription"="Shows a browser selector"
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
