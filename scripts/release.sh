#!/bin/bash
version=`(cargo pkgid | cut -d# -f2 | cut -d: -f2)`
echo $version
mkdir -p temp/target/release;
mkdir -p temp/data;
mkdir -p release;
cargo build --release;
cp target/release/shortcut temp/target/release/shortcut;
cp data/io.github.andreibachim.shortcut.desktop temp/data/io.github.andreibachim.shortcut.desktop;
cp data/io.github.andreibachim.shortcut.metainfo.xml temp/data/io.github.andreibachim.shortcut.metainfo.xml;
cp data/io.github.andreibachim.shortcut.svg temp/data/io.github.andreibachim.shortcut.svg;
zip -r release/v$version.zip temp;
echo The release sha512 is: $(sha512sum release/v$version.zip);
rm -r temp;