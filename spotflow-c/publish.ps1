param (
    [Parameter(Mandatory=$true)]
    [string]$ArtifactsDir,

    [Parameter(Mandatory=$true)]
    [string]$StagingDir,

    [Parameter(Mandatory=$true)]
    [string]$GitHubRef,

    [Parameter(Mandatory=$true)]
    [string]$AccountName,

    [Parameter(Mandatory=$true)]
    [string]$ContainerName
)

# Parse the desired version from the tag

$expectedPrefix = "refs/tags/c-v"

if (!$GitHubRef.StartsWith($expectedPrefix) || $GitHubRef.Length -le $expectedPrefix.Length) {
    Write-Error "Unexpected GitHub ref '$GitHubRef' (expected prefix: '$expectedPrefix')"
    exit 1
}

$version = $GitHubRef.Substring($expectedPrefix.Length)
Write-Output "Version: $version"

# Create the staging directory if it doesn't exist
if (!(Test-Path -PathType Container $StagingDir)) {
    New-Item -ItemType Directory $StagingDir
}

# Pack each directory into a zip file
Get-ChildItem -Path $ArtifactsDir -Directory | ForEach-Object {
    $target = ($_.Name).TrimStart("c-")

    New-Item -ItemType Directory $StagingDir/$target

    Compress-Archive -Path "$($_.FullName)/*" -DestinationPath "$StagingDir/$target/spotflow_device.zip"
}

# Upload the zip files to the storage account

az storage blob upload-batch `
    --auth-mode login `
    --account-name $AccountName `
    --destination $ContainerName `
    --destination-path "device-sdk/$version" `
    --source $StagingDir

az storage blob upload-batch `
    --auth-mode login `
    --account-name $AccountName `
    --destination $ContainerName `
    --destination-path "device-sdk/latest" `
    --source $StagingDir `
    --overwrite
