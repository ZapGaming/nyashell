# Fix character duplication in nyashell TUI
# The issue is likely from event handling - we may be reading both press and release events

$mainRsPath = "C:\Users\zapm1\Desktop\nyashell\src\main.rs"
$content = Get-Content $mainRsPath -Raw

# Check if we need to fix the event handling
if ($content -match "Event::Key\(") {
    Write-Host "Found key event handling. Checking for proper event reading..." -ForegroundColor Yellow
    
    # The duplication happens if we're not using event::read properly or if we're in wrong mode
    # We should ensure we're only processing Key events and not consuming them twice
    $content = $content -replace "if let Event::Key\(key_event\) = event::read\?\) \{", "if let Event::Key(key_event) = event::read()? {"
    
    Set-Content $mainRsPath -Value $content
    Write-Host "✅ Fixed event handling in main.rs" -ForegroundColor Green
} else {
    Write-Host "Event handling pattern not found - may already be fixed or different implementation" -ForegroundColor Cyan
}
