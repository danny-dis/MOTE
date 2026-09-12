$disk = Get-CimInstance Win32_LogicalDisk -Filter 'DeviceID="C:"'
$free = [float]$disk.FreeSpace
$size = [float]$disk.Size
$pct = [math]::Round(($free/$size)*100,2)
$freeGB = [math]::Round($free/1GB,2)
$sizeGB = [math]::Round($size/1GB,2)
Write-Output "Free: $pct% ($freeGB GB / $sizeGB GB)"
if($pct -lt 15) { Write-Output "ALERT: Low disk space" } else { Write-Output "OK" }