#Requires -RunAsAdministrator
# 诊断并修复未打包 WinUI 应用的相机访问权限（ConsentStore webcam）。
# 用法：右键"以管理员身份运行 PowerShell"后执行本脚本。

$ErrorActionPreference = 'Stop'
$base = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\webcam'

Write-Output '== ConsentStore\webcam =='
Get-ItemProperty -Path $base -ErrorAction SilentlyContinue | Select-Object Value, LastSetTime | Format-List

Write-Output '== ConsentStore\webcam\NonPackaged =='
$np = Join-Path $base 'NonPackaged'
if (Test-Path $np) {
    Get-ItemProperty -Path $np | Select-Object Value | Format-List
} else {
    Write-Output 'NonPackaged key MISSING - creating it with Value=Allow'
    New-Item -Path $np -Force | Out-Null
    Set-ItemProperty -Path $np -Name 'Value' -Value 'Allow' -Type String
    Write-Output 'Created NonPackaged = Allow'
}

Write-Output '== Camera devices (PNP) =='
Get-PnpDevice -Class Camera,Image -Status OK -ErrorAction SilentlyContinue |
    Select-Object FriendlyName, InstanceId | Format-Table -AutoSize

Write-Output ''
Write-Output '完成。重新启动 Reactor WinUI Gallery，进入 Capture Element Preview，'
Write-Output '点击 Start preview，并把页面显示的状态文本发回，以便进一步定位。'
