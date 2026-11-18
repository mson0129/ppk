$ErrorActionPreference = 'Stop'

$python = Get-Command py -CommandType Application -ErrorAction SilentlyContinue |
    Select-Object -First 1
$pythonArgs = @('-3')
if (-not $python) {
    $python = Get-Command python -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1
    $pythonArgs = @()
}
if (-not $python) {
    throw 'Install Python 3 and make py or python available on PATH.'
}

Push-Location -LiteralPath $PSScriptRoot
try {
    & $python.Source @pythonArgs tools/package-web.py
    if ($LASTEXITCODE -ne 0) { throw 'Web packaging failed.' }

    & cargo build --locked --release --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { throw 'WASM compilation failed.' }

    Copy-Item -LiteralPath 'target/wasm32-unknown-unknown/release/raccoon_engine.wasm' `
        -Destination 'dist/port/ppk.wasm' -Force

    & $python.Source @pythonArgs tools/build-service-worker.py
    if ($LASTEXITCODE -ne 0) { throw 'Service worker generation failed.' }
}
finally {
    Pop-Location
}
