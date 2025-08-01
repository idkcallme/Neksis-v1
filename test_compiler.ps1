# Test script for Neksis compiler
Write-Host "Testing Neksis compiler..."

# Try to run the compiler with help
try {
    $result = & "C:\Users\renua\.cargo\bin\neksis.exe" --help 2>&1
    Write-Host "Help command result: $result"
} catch {
    Write-Host "Error running help command: $_"
}

# Try to run the compiler with version
try {
    $result = & "C:\Users\renua\.cargo\bin\neksis.exe" --version 2>&1
    Write-Host "Version command result: $result"
} catch {
    Write-Host "Error running version command: $_"
}

# Try to run the compiler with no arguments
try {
    $result = & "C:\Users\renua\.cargo\bin\neksis.exe" 2>&1
    Write-Host "No args command result: $result"
} catch {
    Write-Host "Error running no args command: $_"
}

Write-Host "Test completed." 