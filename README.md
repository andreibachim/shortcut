# Shortcut

## Description

Shortcut is an desktop app made using Rust, GTK4 and Libadawaita that can easily create <code>.desktop</code> files. 
It is specifically designed to visually integrate with the GNOME desktop environment. 

## Installing

<a href="https://flathub.org/apps/io.github.andreibachim.shortcut">
<img src="https://flathub.org/assets/badges/flathub-badge-i-en.png" width="190px" />
</a>


### Build from source code

Requirements
- Minimum Rust version > 1.70 - check it by running: ```rustc -V```
- Minimum GTK version > 4.10 - check it by running ```pkg-config --modversion gtk4```
- Minimium Libadwaita version > 1.3 - check it by running ```pkg-config --modversion libadwaita-1```

## Roadmap

The roadmap for adding features includes:

- Expert mode - create <code>.desktop</code> files following the full [Freedesktop specification](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#recognized-keys)
- Editing existing <code>.desktop</code> files
- Integrate with a translation API to automatically generate localized values for keys
