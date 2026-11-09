#!/bin/sh
set -eu

VERSION=""
BASE_URL=""
OUTPUT_DIR=""
SHA_WINDOWS_X64=""

usage() {
    cat <<'EOF'
Generate a winget manifest bundle for a released yamcli version.

Usage:
  generate-winget-manifests.sh \
    --version VERSION \
    --base-url URL \
    --output-dir DIR \
    --sha256-windows-x64 SHA256
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --base-url)
            BASE_URL="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --sha256-windows-x64)
            SHA_WINDOWS_X64="$2"
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

if [ -z "$VERSION" ] || [ -z "$BASE_URL" ] || [ -z "$OUTPUT_DIR" ] || [ -z "$SHA_WINDOWS_X64" ]; then
    usage >&2
    exit 1
fi

package_id="aikalinin.yamcli"
manifest_root="${OUTPUT_DIR}/${package_id}"
mkdir -p "$manifest_root"

cat > "${manifest_root}/${package_id}.yaml" <<EOF
PackageIdentifier: ${package_id}
PackageVersion: ${VERSION}
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
EOF

cat > "${manifest_root}/${package_id}.installer.yaml" <<EOF
PackageIdentifier: ${package_id}
PackageVersion: ${VERSION}
Platform:
  - Windows.Desktop
InstallModes:
  - interactive
  - silent
  - silentWithProgress
Installers:
  - Architecture: x64
    InstallerType: zip
    NestedInstallerType: portable
    NestedInstallerFiles:
      - RelativeFilePath: yamcli.exe
        PortableCommandAlias: yamcli
    InstallerUrl: ${BASE_URL}/yamcli-x86_64-pc-windows-msvc.zip
    InstallerSha256: ${SHA_WINDOWS_X64}
ManifestType: installer
ManifestVersion: 1.6.0
EOF

cat > "${manifest_root}/${package_id}.locale.en-US.yaml" <<EOF
PackageIdentifier: ${package_id}
PackageVersion: ${VERSION}
PackageLocale: en-US
Publisher: aikalinin
PublisherUrl: https://github.com/aikalinin
PublisherSupportUrl: https://github.com/aikalinin/mcp-local-ya/issues
Author: aikalinin
PackageName: yamcli
PackageUrl: https://github.com/aikalinin/mcp-local-ya
License: MIT
LicenseUrl: https://github.com/aikalinin/mcp-local-ya/blob/v${VERSION}/LICENSE
ShortDescription: Yandex Mail, Calendar, and Disk CLI for humans and AI agents
Description: yamcli is an open-source command-line interface for working with Yandex Mail, Calendar, and Disk from the terminal and from AI agents.
Moniker: yamcli
Tags:
  - cli
  - yandex
  - mail
  - calendar
  - disk
ManifestType: defaultLocale
ManifestVersion: 1.6.0
EOF
