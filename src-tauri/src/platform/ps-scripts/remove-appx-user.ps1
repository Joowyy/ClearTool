$ErrorActionPreference = "Stop"
$pfn = $env:CT_APPX_PFN
if ([string]::IsNullOrWhiteSpace($pfn)) {
    @{ ok = $false; error = "missing CT_APPX_PFN" } | ConvertTo-Json -Compress
    exit 2
}
try {
    $pkg = Get-AppxPackage -ErrorAction Stop | Where-Object { $_.PackageFamilyName -eq $pfn }
    if ($null -eq $pkg) {
        @{ ok = $true; status = "not-installed" } | ConvertTo-Json -Compress
        exit 0
    }
    Remove-AppxPackage -Package $pkg.PackageFullName -ErrorAction Stop
    @{ ok = $true; status = "removed" } | ConvertTo-Json -Compress
} catch {
    @{ ok = $false; error = $_.Exception.Message } | ConvertTo-Json -Compress
    exit 1
}
