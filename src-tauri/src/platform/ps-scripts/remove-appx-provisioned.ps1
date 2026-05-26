$ErrorActionPreference = "Stop"
$name = $env:CT_APPX_PROV_NAME
if ([string]::IsNullOrWhiteSpace($name)) {
    @{ ok = $false; error = "missing CT_APPX_PROV_NAME" } | ConvertTo-Json -Compress
    exit 2
}
try {
    $prov = Get-AppxProvisionedPackage -Online | Where-Object { $_.DisplayName -eq $name }
    if ($null -eq $prov) {
        @{ ok = $true; status = "not-provisioned" } | ConvertTo-Json -Compress
        exit 0
    }
    Remove-AppxProvisionedPackage -Online -PackageName $prov.PackageName | Out-Null
    @{ ok = $true; status = "removed" } | ConvertTo-Json -Compress
} catch {
    @{ ok = $false; error = $_.Exception.Message } | ConvertTo-Json -Compress
    exit 1
}
