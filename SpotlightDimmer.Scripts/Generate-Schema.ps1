# Generate-Schema.ps1
# Regenerates config.schema.json from C# AppConfig class
#
# Usage:
#   .\Generate-Schema.ps1
#
# This script runs the SpotlightDimmer.SchemaGenerator tool to automatically
# generate the JSON schema file from the C# configuration types. This ensures
# the schema stays in sync with the code.
#
# Prerequisites:
#   - .NET 10 SDK

param()

$ErrorActionPreference = "Stop"

Write-Host "`n=========================================" -ForegroundColor Cyan
Write-Host "  JSON Schema Generator" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# Navigate to repository root (script should be in SpotlightDimmer.Scripts)
$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot

try {
    $schemaGeneratorProject = "SpotlightDimmer.SchemaGenerator/SpotlightDimmer.SchemaGenerator.csproj"
    $outputPath = Join-Path $repoRoot "config.schema.json"

    if (-not (Test-Path $schemaGeneratorProject)) {
        Write-Error "Schema generator project not found: $schemaGeneratorProject"
        exit 1
    }

    Write-Host "`n==> Running schema generator..." -ForegroundColor Cyan
    Write-Host "    Project: $schemaGeneratorProject" -ForegroundColor Gray
    Write-Host "    Output: $outputPath`n" -ForegroundColor Gray

    # Run the schema generator tool
    dotnet run --project $schemaGeneratorProject -- $outputPath

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Schema generator failed with exit code $LASTEXITCODE"
        exit $LASTEXITCODE
    }

    Write-Host "`n=========================================" -ForegroundColor Green
    Write-Host "  SCHEMA GENERATED SUCCESSFULLY!" -ForegroundColor Green
    Write-Host "=========================================" -ForegroundColor Green

    $schemaFile = Get-Item $outputPath
    $sizeKB = [math]::Round($schemaFile.Length / 1KB, 2)

    Write-Host "`nSchema file:" -ForegroundColor Cyan
    Write-Host "  Path: $($schemaFile.FullName)" -ForegroundColor Yellow
    Write-Host "  Size: $sizeKB KB" -ForegroundColor Gray
    Write-Host "`nThe schema has been regenerated from the C# AppConfig class." -ForegroundColor Gray
    Write-Host "Commit the updated config.schema.json file if it has changed.`n" -ForegroundColor Gray
}
catch {
    Write-Host "`n=========================================" -ForegroundColor Red
    Write-Host "  SCHEMA GENERATION FAILED!" -ForegroundColor Red
    Write-Host "=========================================" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
finally {
    Pop-Location
}
