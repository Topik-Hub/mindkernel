param(
    [string]$KernelDir = "D:\mindkernel",
    [string]$KernelExe = "D:\mindkernel\target\release\mindkernel.exe",
    [int]$CheckIntervalSec = 5
)

$SignalFile = Join-Path $KernelDir ".rebuild_signal"
$LogFile = Join-Path $KernelDir "watchdog.log"

function Write-Log($msg) {
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    "$timestamp [WATCHDOG] $msg" | Out-File -FilePath $LogFile -Encoding utf8 -Append
    Write-Host "$timestamp [WATCHDOG] $msg"
}

Write-Log "Watchdog started. Monitoring $KernelExe"

while ($true) {
    if (Test-Path $SignalFile) {
        $signal = Get-Content $SignalFile -Raw | ForEach-Object { $_.Trim() }
        Write-Log "Rebuild signal detected: $signal"
        Remove-Item $SignalFile -Force

        # 1. Stop mindkernel
        Write-Log "Stopping mindkernel..."
        $proc = Get-Process -Name "mindkernel" -ErrorAction SilentlyContinue
        if ($proc) {
            $proc | Stop-Process -Force
            Write-Log "Mindkernel stopped (PID $($proc.Id))"
            Start-Sleep -Seconds 2
        } else {
            Write-Log "Mindkernel not running"
        }

        # 2. Check if there's a backup file to restore before building
        $backupFiles = Get-ChildItem -Path "$KernelDir\src\*.bak" -ErrorAction SilentlyContinue
        if ($backupFiles) {
            Write-Log "Found backup files, will restore on build failure"
        }

        # 3. Rebuild
        Write-Log "Building mindkernel..."
        Set-Location -Path $KernelDir
        $buildResult = & cargo build --release 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Log "BUILD FAILED! Restoring backups..."
            # Restore all .bak files to .rs
            foreach ($bak in (Get-ChildItem -Path "$KernelDir\src\*.bak")) {
                $origPath = $bak.FullName -replace '\.bak$', ''
                Copy-Item -Path $bak.FullName -Destination $origPath -Force
                Write-Log "Restored $($bak.Name) → $origPath"
            }
            # Rebuild after restore
            $buildResult = & cargo build --release 2>&1
            if ($LASTEXITCODE -ne 0) {
                Write-Log "BUILD FAILED even after restore! Manual intervention required."
                Start-Sleep -Seconds 60
                continue
            }
        }
        Write-Log "Build successful"

        # 4. Clean up backup files
        Get-ChildItem -Path "$KernelDir\src\*.bak" -ErrorAction SilentlyContinue | Remove-Item -Force
        Write-Log "Backup files cleaned"

        # 5. Start new mindkernel
        Write-Log "Starting new mindkernel..."
        $logDir = Split-Path $LogFile -Parent
        $stdoutLog = Join-Path $logDir "mindkernel_stdout.log"
        Start-Process -FilePath $KernelExe -WindowStyle Hidden -RedirectStandardOutput $stdoutLog
        Write-Log "New mindkernel started"
        Start-Sleep -Seconds 3
    }

    Start-Sleep -Seconds $CheckIntervalSec
}
