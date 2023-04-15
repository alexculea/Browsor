# Browsor: browser selector desktop app for Windows 10

A tool that registers as the default system browser prompting you to select one of the installed ones to open the URL. Works whenever you open a link from any desktop app.

## Screenshot
  ![]( assets/program-screenshot.png )

## Use cases
  - Use multiple browser profiles on the fly, eg: work vs personal
  - Test different browser versions, betas, development, etc
  - Choose everytime based on what web apps are known to work best on, such as Google services in Chrome

## Installing from source
Prerequisites: [Rust compiler](https://www.rust-lang.org/learn/get-started)

### 1. In any PowerShell terminal at the root of the repository, run:

```PowerShell
# from the project root folder, run:
cargo build --release
# Then copy the needed files in a temporary folder
mkdir $env:TMP\Browsor
cp .\target\release\Browsor.exe $env:TMP\Browsor\Browser.exe
cp .\scripts\windows\* $env:TMP\Browsor
# Start an elevated PowerShell terminal at the temp folder
Start-Process Powershell -Verb runAs -ArgumentList '-noexit -command cd $env:TMP\Browsor'
```
Switch to the newly opened **admin powershell** for the rest of the commands

```Powershell
# Allow running unsigned powershell scripts (https:/go.microsoft.com/fwlink/?LinkID=135170)
Set-ExecutionPolicy -ExecutionPolicy Unrestricted
# Runs the install
.\setup.ps1
# For security, revert the policy
Set-ExecutionPolicy -ExecutionPolicy Default
# To uninstall, just run again (needs elevated console):
.\setup.ps1
```

### 2. Set default browser:

Open Settings (🪟 + I) > Apps > Default Apps > Browser selector > Set default

Note: You might need to reboot in order to see the program listed as a browser in the system.
