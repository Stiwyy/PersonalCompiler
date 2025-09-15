#!/usr/bin/env bash
# SPP Compiler RPM Packaging Script
# Date: 17-08-2025
# Author: Stiwyy
set -euo pipefail

# Configurable metadata
PKG_NAME="spp"
VERSION="0.1.0"
RELEASE="1"
SUMMARY="SPP Programming Language Compiler"
DESCRIPTION="A compiler for the SPP programming language.
Compiles .spp files to native executables via NASM."
# You can override LICENSE via env: LICENSE="Apache-2.0" ./package-rpm.sh
LICENSE="${LICENSE:-MIT}"
MAINTAINER="Stiwyy"

# Paths
PROJECT_ROOT="$(pwd)"
BINARY_SOURCE="${PROJECT_ROOT}/target/release/skibidipp"
TOPDIR="${PROJECT_ROOT}/rpmbuild"
SOURCES_DIR="${TOPDIR}/SOURCES"
SPECS_DIR="${TOPDIR}/SPECS"
BUILDROOT_DIR="${TOPDIR}/BUILDROOT"
RPMS_DIR="${TOPDIR}/RPMS"

echo "Creating ${PKG_NAME} ${VERSION}-${RELEASE} RPM package..."

# Check requirements
if ! command -v rpmbuild >/dev/null 2>&1; then
  echo "Error: rpmbuild not found. Install rpm-build (e.g., dnf install rpm-build or apt install rpm) and retry."
  exit 1
fi

if [ ! -f "${BINARY_SOURCE}" ]; then
  echo "Error: Binary not found at ${BINARY_SOURCE}"
  echo "Build it first with: cargo build --release"
  exit 1
fi

# Clean previous artifacts
rm -rf "${TOPDIR}"
mkdir -p "${TOPDIR}"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

# Create staging source tree that matches final filesystem layout
SRC_STAGING="$(mktemp -d -p "$(dirname "${SOURCES_DIR}")" "${PKG_NAME}-${VERSION}.XXXXXXXX")"
PKGROOT="${SRC_STAGING}/${PKG_NAME}-${VERSION}"

mkdir -p "${PKGROOT}/usr/bin"
mkdir -p "${PKGROOT}/usr/share/man/man1"
mkdir -p "${PKGROOT}/usr/share/mime/packages"

echo "Copying skibidipp binary to package..."
install -m 0755 "${BINARY_SOURCE}" "${PKGROOT}/usr/bin/${PKG_NAME}"

# Helper script spp-new
cat > "${PKGROOT}/usr/bin/${PKG_NAME}-new" << 'EOF'
#!/usr/bin/env bash
# Create a new SPP source file
# Created by: Stiwyy
# Date: 17-08-2025

set -euo pipefail

if [ -z "${1-}" ]; then
    echo "Usage: spp-new <filename.spp>"
    exit 1
fi

FILENAME="$1"

if [[ ! "${FILENAME}" =~ \.spp$ ]]; then
    FILENAME="${FILENAME}.spp"
fi

if [ -e "${FILENAME}" ]; then
    echo "Error: File ${FILENAME} already exists"
    exit 1
fi

cat > "${FILENAME}" << 'EOFTEMPLATE'
// SPP Program
// Created: $(date +"%Y-%m-%d %H:%M:%S")
// Author: $(whoami)

// Print hello world
console.print("Hello, World!");

// Exit with success
exit(0);
EOFTEMPLATE

echo "Created new SPP file: ${FILENAME}"
echo "Compile with: spp ${FILENAME}"
echo "Run with: ./$(basename "${FILENAME}" .spp)"
EOF
chmod 0755 "${PKGROOT}/usr/bin/${PKG_NAME}-new"

# MIME type
cat > "${PKGROOT}/usr/share/mime/packages/${PKG_NAME}.xml" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="text/x-spp">
    <comment>SPP source code</comment>
    <glob pattern="*.spp"/>
  </mime-type>
</mime-info>
EOF
chmod 0644 "${PKGROOT}/usr/share/mime/packages/${PKG_NAME}.xml"

# Man page
cat > "${PKGROOT}/usr/share/man/man1/${PKG_NAME}.1" << 'EOF'
.TH SPP 1 "August 2025" "SPP 0.1.0" "SPP Manual"
.SH NAME
spp \- compiler for the SPP programming language
.SH SYNOPSIS
.B spp
[\fIOPTIONS\fR]
\fIFILE\fR
.SH DESCRIPTION
.B spp
compiles SPP language source files to native executables.
.SH OPTIONS
.TP
.BR \-h ", " \-\-help
Display help information
.TP
.BR \-v ", " \-\-version
Display version information
.TP
.BR \-o ", " \-\-output=\fIFILE\fR
Specify output executable name
.TP
.BR \-\-output-dir=\fIDIR\fR
Specify output directory (default: current directory)
.SH EXAMPLES
.TP
Compile a source file:
.B spp program.spp
.TP
Compile with custom output name:
.B spp -o myprogram program.spp
.TP
Run the compiled program:
.B ./program
.SH AUTHOR
Stiwyy
EOF
gzip -9n "${PKGROOT}/usr/share/man/man1/${PKG_NAME}.1"

# Create source tarball
TARBALL_NAME="${PKG_NAME}-${VERSION}.tar.gz"
tar -C "${SRC_STAGING}" -czf "${SOURCES_DIR}/${TARBALL_NAME}" "${PKG_NAME}-${VERSION}"

# Create RPM spec
SPEC_FILE="${SPECS_DIR}/${PKG_NAME}.spec"
cat > "${SPEC_FILE}" << EOF
%global debug_package %{nil}

Name:           ${PKG_NAME}
Version:        ${VERSION}
Release:        ${RELEASE}%{?dist}
Summary:        ${SUMMARY}
License:        ${LICENSE}
URL:            https://github.com/${MAINTAINER}
Source0:        %{name}-%{version}.tar.gz

Requires:       nasm >= 2.13
Requires:       binutils

%description
${DESCRIPTION}

%prep
%setup -q

%build
# No build needed; prebuilt binary

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a usr %{buildroot}/

%post
# Ensure executables have correct permissions
if [ -f /usr/bin/${PKG_NAME} ]; then
    chmod 755 /usr/bin/${PKG_NAME} || true
fi

if [ -f /usr/bin/${PKG_NAME}-new ]; then
    chmod 755 /usr/bin/${PKG_NAME}-new || true
fi

# Update MIME database if available
if command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database /usr/share/mime >/dev/null 2>&1 || true
fi

%files
%defattr(-,root,root,-)
%attr(0755,root,root) /usr/bin/${PKG_NAME}
%attr(0755,root,root) /usr/bin/${PKG_NAME}-new
/usr/share/mime/packages/${PKG_NAME}.xml
/usr/share/man/man1/${PKG_NAME}.1.gz

%changelog
* Mon Aug 18 2025 ${MAINTAINER} <noreply@example.com> - ${VERSION}-${RELEASE}
- Initial RPM release
EOF

# Build RPM
echo "Building RPM..."
rpmbuild -bb --define "_topdir ${TOPDIR}" "${SPEC_FILE}"

# Find resulting RPM
RPM_FILE="$(find "${RPMS_DIR}" -type f -name "${PKG_NAME}-${VERSION}-${RELEASE}*.rpm" | head -n 1 || true)"
if [ -z "${RPM_FILE}" ]; then
  echo "Error: RPM build did not produce an output file."
  exit 1
fi

echo "Package created: ${RPM_FILE}"
echo "Done."