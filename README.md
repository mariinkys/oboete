<div align="center">
  <br>
  <img src="https://raw.githubusercontent.com/mariinkys/oboete/main/res/icons/hicolor/256x256/apps/dev.mariinkys.Oboete.svg" width="150" />
  <h1>Oboete</h1>

  <h3>A simple flashcards application for the COSMIC™ desktop</h3>

  <img alt="Main Window Dark" width="200" src="https://raw.githubusercontent.com/mariinkys/oboete/main/res/screenshots/main-dark.png"/>
  <img alt="Folder Window Dark" width="200" src="https://raw.githubusercontent.com/mariinkys/oboete/main/res/screenshots/folder-inside-dark.png"/>
  <img alt="Study Window Dark" width="200" src="https://raw.githubusercontent.com/mariinkys/oboete/main/res/screenshots/study-dark.png"/>

  <br><br>

  <a href="https://flathub.org/apps/dev.mariinkys.Oboete">
    <img width='240' alt='Download on Flathub' src='https://flathub.org/api/badge?locale=en'/>
  </a>
</div>

# Notes

This application has been made thanks to the [libcosmic Documentation](https://pop-os.github.io/libcosmic/cosmic/) and [edfloreshz](https://github.com/edfloreshz) application template and examples.

This project is related to my [other flashcard project](https://github.com/mariinkys/delphinus_flashcards), if you want Chinese or Japanese flashcards you can [check it out](https://github.com/mariinkys/delphinus_flashcards)!

## Known Issues

- [Flatpak not displaying some Languages (Displaying TOFU characters)](https://github.com/mariinkys/oboete/issues/4)

## Anki Importing Support

Please Look at: [ANKI_IMPORTING](https://github.com/mariinkys/oboete/blob/main/info/ANKI_IMPORTING.md)

# Installation
```
git clone https://github.com/mariinkys/oboete.git
cd oboete
cargo build --release
sudo just install
```

# Development Notes
In order to build the Flatpak, first you need to create the 'cargo-sources.json' file, for that we'll use [this python script, from flatpak-builder-tools](https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo), remember that the 'toml' and 'aiohttp' python modules are needed (they can be installed with pip).

Once you have that, with the python script in the root of the project, you can start with:
```
python3 flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
```
This will create the needed 'cargo-sources.json' file. 
Then you already can build and install the Flatpak with:
```
flatpak-builder --user --install --force-clean build-dir dev.mariinkys.Oboete.json
```
You can also build the Flatpak and not install it with:
```
flatpak-builder --force-clean build-dir dev.mariinkys.Oboete.json
```
Useful resources include:
[Flatpak Docs](https://docs.flatpak.org/en/latest/first-build.html). Remember that whenever the dependencies change/are updated the 'cargo-sources.json' file needs to be rebuilt.

# Copyright and Licensing

Copyright 2024 © Alex Marín

Released under the terms of the [GPL-3.0](https://github.com/mariinkys/oboete/blob/main/LICENSE)
