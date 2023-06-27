# vpxtool
Cross-platform console based utility for the vpinball ecosystem

## Usage

For now you need `rust` on your system

Show help

```
> cargo run -- --help
```

Extract everything

```
> cargo run -- extract ~/path/to/table.vpx
extracting from /Users/me/Downloads/tlk35/tlk-0.35.vpx
Info file written to
  /Users/me/Downloads/tlk35/tlk-0.35/TableInfo.json
VBScript file written to
  /Users/me/Downloads/tlk35/tlk-0.35.vbs
Binaries written to
  /Users/me/Downloads/tlk35/tlk-0.35
```

To only extract the `VBScript`

```
> cargo run -- extractvbs ~/Downloads/tlk35/tlk-0.35.vpx 
extracting from /Users/me/Downloads/tlk35/tlk-0.35.vpx
VBScript file written to
  /Users/me/Downloads/tlk35/tlk-0.35.vbs
```

This will create a folder `~/path/to/table` containing the contents of the `vpx` file.

## References / Research

Other related projects that read assemble vpx files:

* https://github.com/vpinball/vpinball
* https://github.com/vpdb/vpx-js
* https://github.com/freezy/VisualPinball.Engine
* https://github.com/stojy/ClrVpin
* https://github.com/vbousquet/vpx_lightmapper

An example vpx managed in github with some imagemagick scripts to compose textures

https://github.com/vbousquet/flexdmd/tree/master/FlexDemo

