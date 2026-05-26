$ErrorActionPreference = "Stop"

$user = @(Get-AppxPackage -ErrorAction SilentlyContinue | Select-Object Name, PackageFamilyName, InstallLocation, Version)
$provisional = @()
try {
    $provisional = @(Get-AppxProvisionedPackage -Online -ErrorAction Stop | Select-Object DisplayName, PackageName)
} catch {
    # No admin — provisional queda vacío. Es OK.
}

@{
    user        = $user
    provisional = $provisional
} | ConvertTo-Json -Compress -Depth 4
