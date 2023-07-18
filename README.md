# vpxtool
Cross-platform console based utility for the vpinball ecosystem

## Install

Download the latest release for your operating system at https://github.com/francisdb/vpxtool/releases, extract it and if wanted copy or symlink the binary to `$HOME/bin` to put in on your path

## Usage

Show help

```
> vpxtool --help
Extracts and assembles vpx files

Usage: vpxtool [COMMAND]

Commands:
  info        Show information about a vpx file
  diff        Prints out a diff between the vbs in the vpx and the sidecar vbs
  frontend    Acts as a frontend for launching vpx files
  index       Indexes a directory of vpx files
  extract     Extracts a vpx file
  extractvbs  Extracts the vbs from a vpx file next to it
  importvbs   Imports the vbs next to it into a vpx file
  verify      Verify the structure of a vpx file
  assemble    Assembles a vpx file
  new         Creates a minimal empty new vpx file
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Show help for a specific command

```
> vpxtool frontend --help`
Acts as a frontend for launching vpx files

Usage: vpxtool frontend [OPTIONS] [VPXROOTPATH]

Arguments:
  [VPXROOTPATH]  The path to the root directory of vpx files [default: /Users/francisdb/vpinball/tables]

Options:
  -r, --recursive  Recursively index subdirectories
  -h, --help       Print help
```

## References / Research

Other related projects that read assemble vpx files:

* https://github.com/vpinball/vpinball
* https://github.com/vpdb/vpx-js
* https://github.com/freezy/VisualPinball.Engine
* https://github.com/stojy/ClrVpin
* https://github.com/vbousquet/vpx_lightmapper

An example vpx managed in github with some imagemagick scripts to compose textures

https://github.com/vbousquet/flexdmd/tree/master/FlexDemo

