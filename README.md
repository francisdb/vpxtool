# vpxtool
Extracts and assembles visual pinball vpx files

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
