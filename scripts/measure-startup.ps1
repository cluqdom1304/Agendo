<#
  Mide N-01 (arranque hasta la primera ventana visible) y N-02 (memoria en reposo) de la
  versión release. Usa una carpeta de datos temporal vacía (MIAGENDA_DATA_DIR): no toca los
  datos reales. Uso, tras `npm run tauri -- build --no-bundle`:

    powershell -File scripts/measure-startup.ps1 [-Exe ruta] [-Runs 5] [-IdleSeconds 10]
#>
param(
  [string]$Exe = "src-tauri/target/release/agendo.exe",
  [int]$Runs = 5,
  [int]$IdleSeconds = 10
)

$ErrorActionPreference = "Stop"
if (-not (Test-Path $Exe)) { throw "No existe $Exe; compila primero la versión release." }
$Exe = (Resolve-Path $Exe).Path
# La aplicación es de instancia única: una abierta haría que cada arranque medido saliera al instante.
if (Get-Process -Name ([System.IO.Path]::GetFileNameWithoutExtension($Exe)) -ErrorAction SilentlyContinue) {
  throw "Hay una instancia de la aplicación abierta; ciérrala antes de medir."
}

function Get-TreeMemoryMb([int]$RootId) {
  $ids = @($RootId)
  $queue = @($RootId)
  while ($queue.Count -gt 0) {
    $current, $queue = $queue[0], @($queue | Select-Object -Skip 1)
    $children = Get-CimInstance Win32_Process -Filter "ParentProcessId = $current" |
      Select-Object -ExpandProperty ProcessId
    foreach ($child in $children) { $ids += $child; $queue += $child }
  }
  $main = (Get-Process -Id $RootId).WorkingSet64 / 1MB
  $total = ($ids | ForEach-Object { (Get-Process -Id $_ -ErrorAction SilentlyContinue).WorkingSet64 } |
    Measure-Object -Sum).Sum / 1MB
  return @{ Main = $main; Total = $total }
}

$startups = @()
$mains = @()
$totals = @()
for ($run = 1; $run -le $Runs; $run++) {
  $data = Join-Path ([System.IO.Path]::GetTempPath()) ("agendo-measure-" + [guid]::NewGuid())
  New-Item -ItemType Directory -Path $data | Out-Null
  $env:MIAGENDA_DATA_DIR = $data
  $clock = [System.Diagnostics.Stopwatch]::StartNew()
  $process = Start-Process -FilePath $Exe -PassThru
  try {
    while ($process.MainWindowHandle -eq 0) {
      if ($clock.Elapsed.TotalSeconds -gt 30) { throw "La ventana no apareció en 30 s." }
      Start-Sleep -Milliseconds 5
      $process.Refresh()
    }
    $startups += $clock.Elapsed.TotalMilliseconds
    Start-Sleep -Seconds $IdleSeconds
    $memory = Get-TreeMemoryMb $process.Id
    $mains += $memory.Main
    $totals += $memory.Total
    "{0}/{1}: ventana en {2:N0} ms; memoria {3:N1} MB (proceso principal), {4:N1} MB (con WebView2)" -f
      $run, $Runs, $startups[-1], $memory.Main, $memory.Total
  } finally {
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    Get-Process -Name msedgewebview2 -ErrorAction SilentlyContinue |
      Where-Object { $_.Path -and (Get-CimInstance Win32_Process -Filter "ProcessId = $($_.Id)").CommandLine -like "*$data*" } |
      Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
    Remove-Item -Recurse -Force $data -ErrorAction SilentlyContinue
  }
}
Remove-Item Env:MIAGENDA_DATA_DIR

$sorted = $startups | Sort-Object
"N-01 (ventana visible): mediana {0:N0} ms, peor {1:N0} ms (límite 200 ms)" -f $sorted[[int]($sorted.Count / 2)], $sorted[-1]
"N-02 (reposo): proceso principal {0:N1} MB (límite 40 MB); total con WebView2 {1:N1} MB" -f
  ($mains | Measure-Object -Average).Average, ($totals | Measure-Object -Average).Average
