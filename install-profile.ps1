# Install NyaShell profile into Windows Terminal settings

$profilePath = "$env:LOCALAPPDATA\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json"
$altPath = "$env:APPDATA\Microsoft\Windows Terminal\settings.json"

# Determine which settings file to use
if (Test-Path $profilePath) {
    $settingsFile = $profilePath
} elseif (Test-Path $altPath) {
    $settingsFile = $altPath
} else {
    # Create the directory and a basic settings file with proper structure
    $settingsDir = Split-Path $profilePath
    if (!(Test-Path $settingsDir)) {
        New-Item -ItemType Directory -Force -Path $settingsDir | Out-Null
    }
    
    # Create a minimal valid Windows Terminal settings JSON
    $defaultSettings = @{
        profiles = @{
            list = @()
        }
    } | ConvertTo-Json -Depth 10
    
    $defaultSettings | Out-File -FilePath $profilePath -Encoding UTF8
    $settingsFile = $profilePath
    Write-Host "Created new Windows Terminal settings file" -ForegroundColor Yellow
}

# Load the settings
try {
    $profile = Get-Content $settingsFile -Raw | ConvertFrom-Json
} catch {
    Write-Error "Failed to parse Windows Terminal settings. The file may be corrupted."
    exit 1
}

# Load the profile template
$profileTemplatePath = "$PSScriptRoot\windows-terminal-profile.json"
Write-Host "Reading profile from: $profileTemplatePath" -ForegroundColor Yellow
$nyashellProfile = Get-Content $profileTemplatePath -Raw | ConvertFrom-Json
Write-Host "Profile commandline: $($nyashellProfile.commandline)" -ForegroundColor Yellow

# Ensure profiles object exists
if (-not $profile.PSObject.Properties.Name -contains 'profiles') {
    $profile | Add-Member -MemberType NoteProperty -Name "profiles" -Value @{ }
}

# Ensure profiles.list exists and is an array
if (-not $profile.profiles.PSObject.Properties.Name -contains 'list') {
    $profile.profiles | Add-Member -MemberType NoteProperty -Name "list" -Value @()
}

# Force list to be an array
if ($profile.profiles.list -isnot [System.Collections.IList]) {
    $profile.profiles.list = @()
}

# Remove existing NyaShell profile if present
$profile.profiles.list = @($profile.profiles.list | Where-Object { $_.name -notlike "*NyaShell*" })

# Add new profile - ensure it's added as array element
$profile.profiles.list = @($profile.profiles.list + $nyashellProfile)

# Convert back to JSON with proper formatting
$json = $profile | ConvertTo-Json -Depth 10

# Write back to file
$json | Out-File -FilePath $settingsFile -Encoding UTF8

Write-Host "NyaShell profile installed successfully!" -ForegroundColor Green
Write-Host "Location: $settingsFile" -ForegroundColor Cyan
Write-Host "Total profiles: $($profile.profiles.list.Count)" -ForegroundColor Cyan
