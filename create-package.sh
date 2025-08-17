#!/bin/bash
# SPP Compiler Packaging Script
# Date: 17-08-2025
# Author: Stiwyy

echo "Creating SPP compiler package..."

if [ ! -f "target/release/spp" ]; then
    echo "Building SPP compiler..."
    cargo build --release
fi

mkdir -p spp_0.1.0/DEBIAN
mkdir -p spp_0.1.0/usr/bin
mkdir -p spp_0.1.0/usr/share/man/man1
mkdir -p spp_0.1.0/usr/share/applications
mkdir -p spp_0.1.0/usr/share/mime/packages

cp target/release/spp spp_0.1.0/usr/bin/

cat > spp_0.1.0/DEBIAN/control << EOF
Package: spp
Version: 0.1.0
Section: development
Priority: optional
Architecture: amd64
Depends: nasm (>= 2.13), binutils
Maintainer: Stiwyy <Stiwyy@users.noreply.github.com>
Description: SPP Programming Language Compiler
 A compiler for the SPP programming language.
 Compiles .spp files to native executables via NASM.
EOF

cat > spp_0.1.0/usr/bin/spp-new << EOF
#!/bin/bash
# Create a new SPP source file
# Created by: Stiwyy
# Date: 2025-08-17 12:05:09

if [ -z "\$1" ]; then
    echo "Usage: spp-new <filename.spp>"
    exit 1
fi

FILENAME="\$1"

if [[ ! "\$FILENAME" =~ \.spp$ ]]; then
    FILENAME="\$FILENAME.spp"
fi

if [ -e "\$FILENAME" ]; then
    echo "Error: File \$FILENAME already exists"
    exit 1
fi

cat > "\$FILENAME" << 'EOFTEMPLATE'
// SPP Program
// Created: $(date +"%Y-%m-%d %H:%M:%S")
// Author: $(whoami)

// Print hello world
console.print("Hello, World!");

// Exit with success
exit(0);
EOFTEMPLATE

echo "Created new SPP file: \$FILENAME"
echo "Compile with: spp \$FILENAME"
echo "Run with: ./\$(basename \$FILENAME .spp)"
EOF
chmod 755 spp_0.1.0/usr/bin/spp-new

cat > spp_0.1.0/DEBIAN/postinst << EOF
#!/bin/sh
set -e

# Make sure the executables are properly set
chmod 755 /usr/bin/spp
chmod 755 /usr/bin/spp-new

# Update MIME database
if command -v update-mime-database >/dev/null; then
    update-mime-database /usr/share/mime
fi

exit 0
EOF
chmod 755 spp_0.1.0/DEBIAN/postinst

cat > spp_0.1.0/usr/share/mime/packages/spp.xml << EOF
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="text/x-spp">
    <comment>SPP source code</comment>
    <glob pattern="*.spp"/>
  </mime-type>
</mime-info>
EOF

cat > spp_0.1.0/usr/share/man/man1/spp.1 << EOF
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
Stiwyy <your.email@example.com>
EOF

dpkg-deb --build spp_0.1.0

echo "Package created: spp_0.1.0.deb"