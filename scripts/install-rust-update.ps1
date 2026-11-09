param(
 [Parameter(Mandatory=$true)][string]$Stage,
 [Parameter(Mandatory=$true)][string]$Target,
 [Parameter(Mandatory=$true)][int]$TrayPid,
 [Parameter(Mandatory=$true)][int]$UpdaterPid
)
$ErrorActionPreference='Stop'
$names=@('Print-Agent-Rust.exe','Print-Agent-Rust-Core.exe','Print-Settings-Rust.exe','Print-Updater-Rust.exe','scripts/export-excel-pdf.ps1')
$backup=Join-Path $Stage 'backup'
$success=$false
try {
  foreach($name in $names) {
    $src=Join-Path $Stage $name
    if(-not [System.IO.File]::Exists($src)){throw "Thiếu tệp cập nhật: $name"}
  }
  # Đã xác minh SHA trước khi khởi động script. Không thay thế nếu Core đang tồn tại.
  if(Get-Process -Name 'Print-Agent-Rust-Core' -ErrorAction SilentlyContinue){throw 'Print Agent đang hoạt động: hoãn cập nhật'}
  if($TrayPid -gt 0){Stop-Process -Id $TrayPid -ErrorAction SilentlyContinue}
  if($UpdaterPid -gt 0){
    try{Wait-Process -Id $UpdaterPid -Timeout 40 -ErrorAction SilentlyContinue}catch{}
  }
  Start-Sleep -Milliseconds 1200
  New-Item -ItemType Directory -Path $backup -Force|Out-Null
  foreach($name in $names) {
    $existing=Join-Path $Target $name
    $backupPath=Join-Path $backup $name
    if(Test-Path -LiteralPath $existing){
      New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($backupPath)) -Force|Out-Null
      Copy-Item -LiteralPath $existing -Destination $backupPath -Force
    }
  }
  foreach($name in $names) {
    $existing=Join-Path $Target $name
    New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($existing)) -Force|Out-Null
    Copy-Item -LiteralPath (Join-Path $Stage $name) -Destination $existing -Force -ErrorAction Stop
  }
  $success=$true
} catch {
  $reason=$_.Exception.Message
  foreach($name in $names) {
    $old=Join-Path $backup $name
    if(Test-Path -LiteralPath $old) {
      try {Copy-Item -LiteralPath $old -Destination (Join-Path $Target $name) -Force -ErrorAction Stop}catch{}
    }
  }
  try{Add-Content -LiteralPath (Join-Path $Target 'rust-update.log') -Value "Cập nhật thất bại, đã phục hồi các file cũ: $reason"}catch{}
} finally {
  # Tuyệt đối không xóa config.ini, agent-id.txt, template hoặc PDF.
  if($success){
    try {Add-Content -LiteralPath (Join-Path $Target 'rust-update.log') -Value 'Đã cập nhật Rust thành công qua GitHub Release riêng.'}catch{}
  }
  $agent=Join-Path $Target 'Print-Agent-Rust.exe'
  if(Test-Path -LiteralPath $agent){Start-Process -FilePath $agent -WorkingDirectory $Target}
  try{Remove-Item -LiteralPath $Stage -Recurse -Force -ErrorAction SilentlyContinue}catch{}
}
