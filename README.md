# vpxtool

Cross-platform console based utility for the vpinball ecosystem

Join [#vpxtool on "Virtual Pinball Chat" discord](https://discord.gg/eYsvyMu8) for support and questions.

## Install

Download the latest release for your operating system at https://github.com/francisdb/vpxtool/releases, extract it and
if wanted copy or symlink the binary to `$HOME/bin` to put in on your path

### macOS

After extracting the archive you will have to remove the quarantine flag through
`System Settings / Privacy & Security / Allow Anyway button` or on the command line as shown below.

```
xattr -d com.apple.quarantine vpxtool
```

## Usage

Show help

```
> vpxtool --help
Vpxtool v0.16.0

Extracts and assembles vpx files

Usage: vpxtool [COMMAND]

Commands:
  info            Vpx table info related commands
  diff            Prints out a diff between the vbs in the vpx and the sidecar vbs
  frontend        Text based frontend for launching vpx files
  simplefrontend  Simple text based frontend for launching vpx files
  index           Indexes a directory of vpx files
  script          Vpx script code related commands
  ls              Show a vpx file content
  extract         Extracts a vpx file
  extractvbs      Extracts the vbs from a vpx file next to it
  importvbs       Imports the vbs next to it into a vpx file
  verify          Verify the structure of a vpx file
  assemble        Assembles a vpx file
  patch           Applies a VPURemix System patch to a table
  new             Creates a minimal empty new vpx file
  config          Vpxtool related config file
  images          Vpx image related commands
  gamedata        Vpx gamedata related commands
  romname         Prints the PinMAME ROM name from a VPX file
  help            Print this message or the help of the given subcommand(s)

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

## Frontend

Vpxtool can act as a frontend for launching vpx files. It will index a directory of vpx files and then present a menu to
launch them.

![Frontend](docs/frontend.png)

## Configuration

A configuration file will be written to store the Visual Pinball executable location.

To show the current config location use the following command

```
vpxtool config show
```

### Configuring a custom editor

When actions are invoked that open an editor the default editor configured for your system will be used. In case you
want to override this with a specific editor you can add the following line to the config file:

```yaml
# use Visual Studio Code as default editor
editor = "code"
```

## Projects using vpxtool

* https://github.com/syd711/vpin-studio
* https://github.com/mpcarr/aztec-quest
* https://github.com/francisdb/vpinball-example-table-extracted

## References / Research

Other related projects that read assemble vpx files:

* https://github.com/vpinball/vpinball
* https://github.com/vpdb/vpx-js
* https://github.com/freezy/VisualPinball.Engine
* https://github.com/stojy/ClrVpin
* https://github.com/vbousquet/vpx_lightmapper

An example vpx managed in github with some imagemagick scripts to compose textures

https://github.com/vbousquet/flexdmd/tree/master/FlexDemo

## Building

The project uses the default [rust](https://www.rust-lang.org/) build tool `cargo`. To get going read the docs on
installation and first steps at https://doc.rust-lang.org/cargo/

`cargo build --release`
