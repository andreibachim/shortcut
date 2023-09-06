# Shortcut

## Description

Shortcut is an desktop app made using Rust, GTK4 and Libadawaita that can easily create <code>.desktop</code> files. 
It is specifically designed to visually integrate with the GNOME desktop environment. 

## WIP Notice

Note the the app is under heavy development. I do not recommend using it unless you understand what it does.

(It saves a new file to <code>$HOME/.local/share/applications</code>)

## How to edit an already create app icon

```bash
cd ~/.local/share/applications
vi <your-app>.desktop
```
You can edit the detail of your application icon here, Save and exit to apply change.

## Installing

There is no way to easily install the app right now, you have to build it yourself. 

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
