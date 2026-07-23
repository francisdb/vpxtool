# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.33.7](https://github.com/francisdb/vpxtool/compare/v0.33.6...v0.33.7) - 2026-07-23

### Other

- update Cargo.lock dependencies

## [0.33.6](https://github.com/francisdb/vpxtool/compare/v0.33.5...v0.33.6) - 2026-07-18

### Fixed

- remove unsupported dependabot versioning-strategy ([#837](https://github.com/francisdb/vpxtool/pull/837))
- exit non-zero when aborting at an overwrite prompt ([#828](https://github.com/francisdb/vpxtool/pull/828))

### Other

- link dependabot issue tracking the rejected increase value ([#839](https://github.com/francisdb/vpxtool/pull/839))
- *(deps)* bump clap from 4.6.1 to 4.6.2 ([#838](https://github.com/francisdb/vpxtool/pull/838))
- set dependabot cargo versioning-strategy to increase ([#835](https://github.com/francisdb/vpxtool/pull/835))
- require vpin 0.26.10 ([#836](https://github.com/francisdb/vpxtool/pull/836))
- *(deps)* bump toml_edit ([#833](https://github.com/francisdb/vpxtool/pull/833))
- *(deps)* bump toml from 1.1.2+spec-1.1.0 to 1.1.3+spec-1.1.0 ([#831](https://github.com/francisdb/vpxtool/pull/831))
- *(deps)* bump globset from 0.4.18 to 0.4.19 ([#829](https://github.com/francisdb/vpxtool/pull/829))

## [0.33.5](https://github.com/francisdb/vpxtool/compare/v0.33.4...v0.33.5) - 2026-07-14

### Other

- *(deps)* bump indicatif from 0.18.5 to 0.18.6 ([#822](https://github.com/francisdb/vpxtool/pull/822))
- *(deps)* bump crossbeam-epoch to 0.9.20 (RUSTSEC-2026-0204)
- fix clippy useless_borrows_in_formatting warnings

## [0.33.4](https://github.com/francisdb/vpxtool/compare/v0.33.3...v0.33.4) - 2026-07-06

### Added

- *(cli)* add capture command for playfield screenshots ([#815](https://github.com/francisdb/vpxtool/pull/815))

### Other

- *(deps)* bump indicatif from 0.18.4 to 0.18.5 ([#820](https://github.com/francisdb/vpxtool/pull/820))
- *(deps)* bump rand from 0.10.1 to 0.10.2 ([#818](https://github.com/francisdb/vpxtool/pull/818))
- *(deps)* bump console from 0.16.3 to 0.16.4 ([#817](https://github.com/francisdb/vpxtool/pull/817))
- *(deps)* bump pinmame-nvram from 0.4.12 to 0.4.13 ([#819](https://github.com/francisdb/vpxtool/pull/819))
- *(deps)* bump directb2s from 0.1.1 to 0.1.2 ([#821](https://github.com/francisdb/vpxtool/pull/821))
- *(deps)* bump env_logger from 0.11.10 to 0.11.11 ([#816](https://github.com/francisdb/vpxtool/pull/816))
- *(deps)* bump actions/checkout from 6 to 7 ([#812](https://github.com/francisdb/vpxtool/pull/812))

## [0.33.3](https://github.com/francisdb/vpxtool/compare/v0.33.2...v0.33.3) - 2026-06-08

### Other

- *(deps)* bump log from 0.4.30 to 0.4.32 ([#803](https://github.com/francisdb/vpxtool/pull/803))
- *(deps)* bump chrono from 0.4.44 to 0.4.45 ([#804](https://github.com/francisdb/vpxtool/pull/804))
- *(deps)* bump pinmame-nvram from 0.4.11 to 0.4.12 ([#805](https://github.com/francisdb/vpxtool/pull/805))

## [0.33.2](https://github.com/francisdb/vpxtool/compare/v0.33.1...v0.33.2) - 2026-06-02

### Other

- *(deps)* bump toml_edit ([#802](https://github.com/francisdb/vpxtool/pull/802))
- *(deps)* bump log from 0.4.29 to 0.4.30 ([#800](https://github.com/francisdb/vpxtool/pull/800))
- *(deps)* bump pinmame-nvram from 0.4.10 to 0.4.11 ([#801](https://github.com/francisdb/vpxtool/pull/801))
- *(deps)* bump vpin from 0.26.3 to 0.26.4 ([#799](https://github.com/francisdb/vpxtool/pull/799))
- *(deps)* bump serde_json from 1.0.149 to 1.0.150 ([#797](https://github.com/francisdb/vpxtool/pull/797))

## [0.33.1](https://github.com/francisdb/vpxtool/compare/v0.33.0...v0.33.1) - 2026-05-20

### Other

- *(deps)* disable image default features to drop AV1/AVIF encoder ([#795](https://github.com/francisdb/vpxtool/pull/795))

## [0.33.0](https://github.com/francisdb/vpxtool/compare/v0.32.0...v0.33.0) - 2026-05-20

### Added

- *(cli)* [**breaking**] extract -o/--output-dir, single path only

### Other

- *(cli)* reword top-level help/README header

## [0.32.0](https://github.com/francisdb/vpxtool/compare/v0.31.1...v0.32.0) - 2026-05-18

### Added

- *(scores)* [**breaking**] document scores command in README
- *(scores)* VPReg HighScoreName<N> + longer EMHS character names ([#792](https://github.com/francisdb/vpxtool/pull/792))
- *(scores)* support legacy EM hiscore/hsa1/2/3 in VPReg.ini ([#789](https://github.com/francisdb/vpxtool/pull/789))
- *(scores)* multi-section-key VPReg lookup ([#788](https://github.com/francisdb/vpxtool/pull/788))
- *(scores)* support single-hisc EM tables (no initials) ([#787](https://github.com/francisdb/vpxtool/pull/787))
- *(cli)* scores show falls back to EM-style .txt high score files ([#785](https://github.com/francisdb/vpxtool/pull/785))
- *(cli)* scores show falls back to VPReg then GLF for rom-less .vpx tables ([#784](https://github.com/francisdb/vpxtool/pull/784))
- *(cli)* scores show falls back to VPReg.ini for rom-less .vpx tables ([#783](https://github.com/francisdb/vpxtool/pull/783))
- *(cli)* locale-aware thousands separator and aligned columns in scores show ([#782](https://github.com/francisdb/vpxtool/pull/782))
- *(cli)* add --format pinemhi section layout to scores show ([#781](https://github.com/francisdb/vpxtool/pull/781))
- *(cli)* add `scores show` for PinMAME tables ([#779](https://github.com/francisdb/vpxtool/pull/779))

### Fixed

- *(scores)* case-insensitive VPReg keys + Dim-style variable assignments ([#791](https://github.com/francisdb/vpxtool/pull/791))
- *(scores)* tolerate non-UTF-8 .txt files in the EMHS glob ([#786](https://github.com/francisdb/vpxtool/pull/786))

### Other

- -text for tests/fixtures so CRLF survives in cached blobs
- *(scores)* real-file regression fixtures in tests/fixtures
- *(deps)* bump vpin from 0.26.1 to 0.26.3 ([#790](https://github.com/francisdb/vpxtool/pull/790))

## [0.31.1](https://github.com/francisdb/vpxtool/compare/v0.31.0...v0.31.1) - 2026-05-15

### Fixed

- *(config)* tolerate a broken vpinball ini instead of panicking ([#777](https://github.com/francisdb/vpxtool/pull/777))

## [0.31.0](https://github.com/francisdb/vpxtool/compare/v0.30.0...v0.31.0) - 2026-05-13

### Added

- *(indexer)* detect altsound, altcolor and pup packs per table ([#774](https://github.com/francisdb/vpxtool/pull/774))
- *(cli)* add gameitems list subcommand ([#771](https://github.com/francisdb/vpxtool/pull/771))
- *(cli)* add materials list subcommand ([#772](https://github.com/francisdb/vpxtool/pull/772))
- *(cli)* add collections list subcommand ([#773](https://github.com/francisdb/vpxtool/pull/773))
- *(cli)* add sounds list subcommand ([#770](https://github.com/francisdb/vpxtool/pull/770))
- *(cli)* add images list subcommand ([#769](https://github.com/francisdb/vpxtool/pull/769))
- *(cli)* add nvram show subcommand ([#768](https://github.com/francisdb/vpxtool/pull/768))
- *(cli)* add export vpxz subcommand for mobile transfer ([#766](https://github.com/francisdb/vpxtool/pull/766))

### Other

- *(indexer)* cache vpx_parent read_dir for asset detection ([#775](https://github.com/francisdb/vpxtool/pull/775))
- *(deps)* bump toml_edit ([#765](https://github.com/francisdb/vpxtool/pull/765))

## [0.30.0](https://github.com/francisdb/vpxtool/compare/v0.29.3...v0.30.0) - 2026-05-10

### Added

- *(cli)* add export obj/gltf subcommands ([#763](https://github.com/francisdb/vpxtool/pull/763))

### Other

- *(deps)* [**breaking**] bump vpin to 0.26.0 ([#762](https://github.com/francisdb/vpxtool/pull/762))

## [0.29.3](https://github.com/francisdb/vpxtool/compare/v0.29.2...v0.29.3) - 2026-05-05

### Other

- restore table position after launch and clear screen on normal exit ([#760](https://github.com/francisdb/vpxtool/pull/760))

## [0.29.2](https://github.com/francisdb/vpxtool/compare/v0.29.1...v0.29.2) - 2026-05-04

### Added

- *(cli)* add lock / unlock / lock-status commands ([#317](https://github.com/francisdb/vpxtool/pull/317)) ([#758](https://github.com/francisdb/vpxtool/pull/758))

### Fixed

- *(config)* preserve comments and layout in rewrite_vpx_config ([#759](https://github.com/francisdb/vpxtool/pull/759))
- *(config)* resolve vpinball ini at the modern SDL pref path ([#757](https://github.com/francisdb/vpxtool/pull/757))
- drop deprecated -*TrueFullscreen flags from defaults ([#730](https://github.com/francisdb/vpxtool/pull/730)) ([#755](https://github.com/francisdb/vpxtool/pull/755))

## [0.29.1](https://github.com/francisdb/vpxtool/compare/v0.29.0...v0.29.1) - 2026-05-04

### Other

- *(indexer)* parallelise mtime stats in recursive walk ([#754](https://github.com/francisdb/vpxtool/pull/754))
- parallelize directory walk and reduce NAS round-trips ([#752](https://github.com/francisdb/vpxtool/pull/752))
- *(deps)* bump pinmame-nvram from 0.4.7 to 0.4.8 ([#750](https://github.com/francisdb/vpxtool/pull/750))
- *(deps)* bump vpin from 0.23.5 to 0.23.6 ([#751](https://github.com/francisdb/vpxtool/pull/751))

## [0.29.0](https://github.com/francisdb/vpxtool/compare/v0.28.1...v0.29.0) - 2026-05-03

### Other

- *(indexer)* single-pass index path normalization ([#749](https://github.com/francisdb/vpxtool/pull/749))
- store index paths as relative and normalized ([#743](https://github.com/francisdb/vpxtool/pull/743))

## [0.28.1](https://github.com/francisdb/vpxtool/compare/v0.28.0...v0.28.1) - 2026-05-03

### Fixed

- *(indexer)* hand-roll atomic index write resilient to NAS fsync limits ([#744](https://github.com/francisdb/vpxtool/pull/744))

### Other

- *(frontend)* take configured_pinmame_folder as a parameter ([#746](https://github.com/francisdb/vpxtool/pull/746))
- avoid duplicate vpinball ini reads ([#733](https://github.com/francisdb/vpxtool/pull/733))
- sort index entries by path instead of table name ([#742](https://github.com/francisdb/vpxtool/pull/742))
- use BTreeMap for deterministic properties serialization ([#741](https://github.com/francisdb/vpxtool/pull/741))

## [0.28.0](https://github.com/francisdb/vpxtool/compare/v0.27.2...v0.28.0) - 2026-05-02

### Added

- add configurable tables scan max depth ([#735](https://github.com/francisdb/vpxtool/pull/735))

### Fixed

- *(indexer)* write index_json atomically via temp + rename ([#739](https://github.com/francisdb/vpxtool/pull/739))
- *(indexer)* flush BufWriter explicitly when writing index json ([#737](https://github.com/francisdb/vpxtool/pull/737))

### Other

- *(indexer)* cover tables_scan_max_depth in find_vpx_files ([#740](https://github.com/francisdb/vpxtool/pull/740))
- *(indexer)* drop redundant leading wildcard in game-name regexes ([#732](https://github.com/francisdb/vpxtool/pull/732))
- use regex caching for game name and pinmame detection ([#734](https://github.com/francisdb/vpxtool/pull/734))
- use buffered i/o to avoid nas round-trips ([#736](https://github.com/francisdb/vpxtool/pull/736))

## [0.27.2](https://github.com/francisdb/vpxtool/compare/v0.27.1...v0.27.2) - 2026-04-27

### Other

- *(deps)* bump pinmame-nvram from 0.4.6 to 0.4.7 ([#728](https://github.com/francisdb/vpxtool/pull/728))
- *(deps)* bump rayon from 1.11.0 to 1.12.0 ([#727](https://github.com/francisdb/vpxtool/pull/727))
- *(deps)* bump clap from 4.6.0 to 4.6.1 ([#726](https://github.com/francisdb/vpxtool/pull/726))

## [0.27.1](https://github.com/francisdb/vpxtool/compare/v0.27.0...v0.27.1) - 2026-04-20

### Other

- cargo fmt
- drop shellexpand, rely on shell tilde expansion
- *(deps)* bump rand from 0.10.0 to 0.10.1 ([#724](https://github.com/francisdb/vpxtool/pull/724))
- *(deps)* bump softprops/action-gh-release from 2 to 3 ([#723](https://github.com/francisdb/vpxtool/pull/723))
- *(deps)* bump toml from 1.1.0+spec-1.1.0 to 1.1.2+spec-1.1.0 ([#721](https://github.com/francisdb/vpxtool/pull/721))

## [0.27.0](https://github.com/francisdb/vpxtool/compare/v0.26.3...v0.27.0) - 2026-04-05

### Added

- add vpinball_config to launch templates ([#720](https://github.com/francisdb/vpxtool/pull/720))
- sort index ([#718](https://github.com/francisdb/vpxtool/pull/718))

### Other

- add tests for index sorting ([#719](https://github.com/francisdb/vpxtool/pull/719))
- *(deps)* bump env_logger from 0.11.9 to 0.11.10 ([#716](https://github.com/francisdb/vpxtool/pull/716))
- *(deps)* bump toml from 1.0.7+spec-1.1.0 to 1.1.0+spec-1.1.0 ([#717](https://github.com/francisdb/vpxtool/pull/717))
- *(deps)* bump toml from 1.0.6+spec-1.1.0 to 1.0.7+spec-1.1.0 ([#713](https://github.com/francisdb/vpxtool/pull/713))
- *(deps)* bump vpin from 0.23.3 to 0.23.5 ([#714](https://github.com/francisdb/vpxtool/pull/714))

## [0.26.3](https://github.com/francisdb/vpxtool/compare/v0.26.2...v0.26.3) - 2026-03-16

### Other

- *(deps)* update rand and vpin dependencies to latest versions ([#712](https://github.com/francisdb/vpxtool/pull/712))
- *(deps)* bump console from 0.16.2 to 0.16.3 ([#708](https://github.com/francisdb/vpxtool/pull/708))
- *(deps)* bump Swatinem/rust-cache from 2.8.2 to 2.9.1 ([#707](https://github.com/francisdb/vpxtool/pull/707))
- *(deps)* bump image from 0.25.9 to 0.25.10 ([#709](https://github.com/francisdb/vpxtool/pull/709))
- *(deps)* bump clap from 4.5.60 to 4.6.0 ([#710](https://github.com/francisdb/vpxtool/pull/710))

## [0.26.2](https://github.com/francisdb/vpxtool/compare/v0.26.1...v0.26.2) - 2026-03-11

### Added

- unify artifact names for different platforms ([#704](https://github.com/francisdb/vpxtool/pull/704))

## [0.26.1](https://github.com/francisdb/vpxtool/compare/v0.26.0...v0.26.1) - 2026-03-09

### Other

- *(deps)* bump toml from 1.0.3+spec-1.1.0 to 1.0.6+spec-1.1.0 ([#701](https://github.com/francisdb/vpxtool/pull/701))
- *(deps)* bump pinmame-nvram from 0.4.4 to 0.4.6 ([#700](https://github.com/francisdb/vpxtool/pull/700))

## [0.26.0](https://github.com/francisdb/vpxtool/compare/v0.25.0...v0.26.0) - 2026-03-02

### Added

- add diff to config ([#698](https://github.com/francisdb/vpxtool/pull/698))

### Other

- *(deps)* bump shellexpand from 3.1.1 to 3.1.2 ([#695](https://github.com/francisdb/vpxtool/pull/695))
- *(deps)* bump chrono from 0.4.43 to 0.4.44 ([#694](https://github.com/francisdb/vpxtool/pull/694))
- *(deps)* bump actions/upload-artifact from 6 to 7 ([#693](https://github.com/francisdb/vpxtool/pull/693))
- *(deps)* bump vpin from 0.21.1 to 0.23.2 ([#696](https://github.com/francisdb/vpxtool/pull/696))

## [0.25.0](https://github.com/francisdb/vpxtool/compare/v0.24.15...v0.25.0) - 2026-02-23

### Added

- [**breaking**] config file location changed
- correct config location and document ([#691](https://github.com/francisdb/vpxtool/pull/691))

### Other

- vpin 0.21.1 ([#692](https://github.com/francisdb/vpxtool/pull/692))
- *(deps)* bump toml from 0.9.11+spec-1.1.0 to 1.0.3+spec-1.1.0 ([#688](https://github.com/francisdb/vpxtool/pull/688))
- *(deps)* bump env_logger from 0.11.8 to 0.11.9 ([#687](https://github.com/francisdb/vpxtool/pull/687))
- *(deps)* bump clap from 4.5.58 to 4.5.60 ([#689](https://github.com/francisdb/vpxtool/pull/689))
- improve error message for ini file loading
- *(deps)* bump indicatif from 0.18.3 to 0.18.4 ([#684](https://github.com/francisdb/vpxtool/pull/684))
- *(deps)* bump clap from 4.5.57 to 4.5.58 ([#683](https://github.com/francisdb/vpxtool/pull/683))

## [0.24.15](https://github.com/francisdb/vpxtool/compare/v0.24.14...v0.24.15) - 2026-02-06

### Other

- *(deps)* bump vpin from 0.20.14 to 0.20.15 ([#678](https://github.com/francisdb/vpxtool/pull/678))

## [0.24.14](https://github.com/francisdb/vpxtool/compare/v0.24.13...v0.24.14) - 2026-02-06

### Other

- *(deps)* bump regex from 1.12.2 to 1.12.3 ([#674](https://github.com/francisdb/vpxtool/pull/674))
- switch to trusted publishing

## [0.24.13](https://github.com/francisdb/vpxtool/compare/v0.24.12...v0.24.13) - 2026-02-06

### Other

- *(deps)* bump clap from 4.5.56 to 4.5.57 ([#675](https://github.com/francisdb/vpxtool/pull/675))
- *(deps)* directb2s was split off from vpin ([#676](https://github.com/francisdb/vpxtool/pull/676))
- *(deps)* bump bytes from 1.11.0 to 1.11.1 ([#672](https://github.com/francisdb/vpxtool/pull/672))
- *(deps)* bump clap from 4.5.55 to 4.5.56 ([#670](https://github.com/francisdb/vpxtool/pull/670))
- *(deps)* bump vpin from 0.20.10 to 0.20.11 ([#669](https://github.com/francisdb/vpxtool/pull/669))

## [0.24.12](https://github.com/francisdb/vpxtool/compare/v0.24.11...v0.24.12) - 2026-01-28

### Other

- *(deps)* bump vpin from 0.20.9 to 0.20.10 ([#668](https://github.com/francisdb/vpxtool/pull/668))
- *(deps)* bump vpin from 0.20.3 to 0.20.9 ([#666](https://github.com/francisdb/vpxtool/pull/666))
- *(deps)* bump clap from 4.5.54 to 4.5.55 ([#667](https://github.com/francisdb/vpxtool/pull/667))
- *(deps)* bump colored from 3.0.0 to 3.1.1 ([#663](https://github.com/francisdb/vpxtool/pull/663))
- *(deps)* bump chrono from 0.4.42 to 0.4.43 ([#662](https://github.com/francisdb/vpxtool/pull/662))

## [0.24.11](https://github.com/francisdb/vpxtool/compare/v0.24.10...v0.24.11) - 2026-01-14

### Other

- *(deps)* bump vpin from 0.20.1 to 0.20.3 ([#661](https://github.com/francisdb/vpxtool/pull/661))
- *(deps)* bump toml from 0.9.10+spec-1.1.0 to 0.9.11+spec-1.1.0 ([#658](https://github.com/francisdb/vpxtool/pull/658))
- *(deps)* bump serde_json from 1.0.148 to 1.0.149 ([#659](https://github.com/francisdb/vpxtool/pull/659))
- link to vpx-editor
- *(deps)* bump clap from 4.5.53 to 4.5.54 ([#656](https://github.com/francisdb/vpxtool/pull/656))

## [0.24.10](https://github.com/francisdb/vpxtool/compare/v0.24.9...v0.24.10) - 2026-01-02

### Other

- *(deps)* bump vpin from 0.20.0 to 0.20.1 ([#654](https://github.com/francisdb/vpxtool/pull/654))

## [0.24.9](https://github.com/francisdb/vpxtool/compare/v0.24.8...v0.24.9) - 2025-12-31

### Other

- *(deps)* bump vpin from 0.19.1 to 0.20.0 ([#653](https://github.com/francisdb/vpxtool/pull/653))
- *(deps)* bump serde_json from 1.0.145 to 1.0.148 ([#652](https://github.com/francisdb/vpxtool/pull/652))
- *(deps)* bump console from 0.16.1 to 0.16.2 ([#651](https://github.com/francisdb/vpxtool/pull/651))
- *(deps)* bump toml from 0.9.8 to 0.9.10+spec-1.1.0 ([#650](https://github.com/francisdb/vpxtool/pull/650))
- *(deps)* bump vpin from 0.19.0 to 0.19.1 ([#649](https://github.com/francisdb/vpxtool/pull/649))
- *(deps)* bump pinmame-nvram from 0.4.3 to 0.4.4 ([#648](https://github.com/francisdb/vpxtool/pull/648))
- *(deps)* bump actions/upload-artifact from 5 to 6 ([#645](https://github.com/francisdb/vpxtool/pull/645))
- use rust-cache for clippy

## [0.24.8](https://github.com/francisdb/vpxtool/compare/v0.24.7...v0.24.8) - 2025-12-14

### Other

- *(deps)* bump vpin from 0.18.7 to 0.19.0 ([#643](https://github.com/francisdb/vpxtool/pull/643))

## [0.24.7](https://github.com/francisdb/vpxtool/compare/v0.24.6...v0.24.7) - 2025-12-11

### Other

- *(deps)* bump vpin from 0.18.6 to 0.18.7 ([#642](https://github.com/francisdb/vpxtool/pull/642))
- *(deps)* bump log from 0.4.28 to 0.4.29 ([#640](https://github.com/francisdb/vpxtool/pull/640))
- *(deps)* bump Swatinem/rust-cache from 2.8.1 to 2.8.2 ([#639](https://github.com/francisdb/vpxtool/pull/639))
- *(deps)* bump image from 0.25.8 to 0.25.9 ([#636](https://github.com/francisdb/vpxtool/pull/636))
- *(deps)* bump clap from 4.5.51 to 4.5.53 ([#635](https://github.com/francisdb/vpxtool/pull/635))
- *(deps)* bump indicatif from 0.18.2 to 0.18.3 ([#634](https://github.com/francisdb/vpxtool/pull/634))
- *(deps)* bump actions/checkout from 5 to 6 ([#637](https://github.com/francisdb/vpxtool/pull/637))

## [0.24.6](https://github.com/francisdb/vpxtool/compare/v0.24.5...v0.24.6) - 2025-11-07

### Other

- *(deps)* bump vpin from 0.18.3 to 0.18.6 ([#632](https://github.com/francisdb/vpxtool/pull/632))

## [0.24.5](https://github.com/francisdb/vpxtool/compare/v0.24.4...v0.24.5) - 2025-11-07

### Fixed

- diff for a local path ([#629](https://github.com/francisdb/vpxtool/pull/629))

### Other

- *(deps)* bump indicatif from 0.18.1 to 0.18.2 ([#627](https://github.com/francisdb/vpxtool/pull/627))
- *(deps)* bump clap from 4.5.50 to 4.5.51 ([#626](https://github.com/francisdb/vpxtool/pull/626))
- non-expiring discord invite link
- *(deps)* bump vpin from 0.18.1 to 0.18.3 ([#622](https://github.com/francisdb/vpxtool/pull/622))
- *(deps)* bump indicatif from 0.18.0 to 0.18.1 ([#623](https://github.com/francisdb/vpxtool/pull/623))
- *(deps)* bump clap from 4.5.49 to 4.5.50 ([#624](https://github.com/francisdb/vpxtool/pull/624))
- *(deps)* bump actions/upload-artifact from 4 to 5 ([#625](https://github.com/francisdb/vpxtool/pull/625))
- *(deps)* bump clap from 4.5.48 to 4.5.49 ([#620](https://github.com/francisdb/vpxtool/pull/620))
- *(deps)* bump regex from 1.12.1 to 1.12.2 ([#621](https://github.com/francisdb/vpxtool/pull/621))
- *(deps)* bump regex from 1.11.3 to 1.12.1 ([#618](https://github.com/francisdb/vpxtool/pull/618))

## [0.24.4](https://github.com/francisdb/vpxtool/compare/v0.24.3...v0.24.4) - 2025-10-10

### Added

- *(frontend)* show dip switch info if available ([#617](https://github.com/francisdb/vpxtool/pull/617))

### Other

- *(deps)* bump toml from 0.9.7 to 0.9.8 ([#615](https://github.com/francisdb/vpxtool/pull/615))
- *(deps)* bump pinmame-nvram from 0.4.1 to 0.4.3 ([#614](https://github.com/francisdb/vpxtool/pull/614))

## [0.24.3](https://github.com/francisdb/vpxtool/compare/v0.24.2...v0.24.3) - 2025-10-01

### Added

- standard env logs if RUST_LOG is set

### Other

- document logging
- reverted custom vpin
- verbosity flag added
- *(deps)* bump toml from 0.9.5 to 0.9.7 ([#610](https://github.com/francisdb/vpxtool/pull/610))
- *(deps)* bump regex from 1.11.2 to 1.11.3 ([#609](https://github.com/francisdb/vpxtool/pull/609))
- *(deps)* bump serde from 1.0.223 to 1.0.228 ([#611](https://github.com/francisdb/vpxtool/pull/611))
- *(deps)* bump clap from 4.5.47 to 4.5.48 ([#612](https://github.com/francisdb/vpxtool/pull/612))
- *(deps)* bump Swatinem/rust-cache from 2.8.0 to 2.8.1 ([#606](https://github.com/francisdb/vpxtool/pull/606))
- fix lock file
- *(deps)* bump console from 0.16.0 to 0.16.1 ([#603](https://github.com/francisdb/vpxtool/pull/603))
- *(deps)* bump chrono from 0.4.41 to 0.4.42 ([#604](https://github.com/francisdb/vpxtool/pull/604))
- *(deps)* bump serde_json from 1.0.143 to 1.0.145 ([#605](https://github.com/francisdb/vpxtool/pull/605))
- *(deps)* bump log from 0.4.27 to 0.4.28 ([#597](https://github.com/francisdb/vpxtool/pull/597))
- *(deps)* bump image from 0.25.7 to 0.25.8 ([#596](https://github.com/francisdb/vpxtool/pull/596))
- *(deps)* bump clap from 4.5.46 to 4.5.47 ([#595](https://github.com/francisdb/vpxtool/pull/595))

## [0.24.2](https://github.com/francisdb/vpxtool/compare/v0.24.1...v0.24.2) - 2025-09-03

### Fixed

- search ansi escape issues ([#594](https://github.com/francisdb/vpxtool/pull/594))

### Other

- *(deps)* bump image from 0.25.6 to 0.25.7 ([#590](https://github.com/francisdb/vpxtool/pull/590))
- *(deps)* bump is_executable from 1.0.4 to 1.0.5 ([#591](https://github.com/francisdb/vpxtool/pull/591))
- *(deps)* bump rust-ini from 0.21.2 to 0.21.3 ([#592](https://github.com/francisdb/vpxtool/pull/592))
- *(deps)* bump clap from 4.5.45 to 4.5.46 ([#589](https://github.com/francisdb/vpxtool/pull/589))
- *(deps)* bump dialoguer from 0.11.0 to 0.12.0 ([#588](https://github.com/francisdb/vpxtool/pull/588))
- *(deps)* bump serde_json from 1.0.142 to 1.0.143 ([#587](https://github.com/francisdb/vpxtool/pull/587))
- *(deps)* bump regex from 1.11.1 to 1.11.2 ([#586](https://github.com/francisdb/vpxtool/pull/586))
- *(deps)* bump actions/checkout from 4 to 5 ([#584](https://github.com/francisdb/vpxtool/pull/584))
- *(deps)* bump rayon from 1.10.0 to 1.11.0 ([#585](https://github.com/francisdb/vpxtool/pull/585))
- *(deps)* bump toml from 0.9.4 to 0.9.5 ([#582](https://github.com/francisdb/vpxtool/pull/582))
- *(deps)* bump clap from 4.5.42 to 4.5.43 ([#581](https://github.com/francisdb/vpxtool/pull/581))
- new clippy rules ([#583](https://github.com/francisdb/vpxtool/pull/583))
- *(deps)* bump serde_json from 1.0.141 to 1.0.142 ([#578](https://github.com/francisdb/vpxtool/pull/578))
- *(deps)* bump clap from 4.5.41 to 4.5.42 ([#579](https://github.com/francisdb/vpxtool/pull/579))
- *(deps)* bump toml from 0.9.2 to 0.9.4 ([#580](https://github.com/francisdb/vpxtool/pull/580))
- *(deps)* bump rand from 0.9.1 to 0.9.2 ([#575](https://github.com/francisdb/vpxtool/pull/575))
- *(deps)* bump serde_json from 1.0.140 to 1.0.141 ([#576](https://github.com/francisdb/vpxtool/pull/576))

## [0.24.1](https://github.com/francisdb/vpxtool/compare/v0.24.0...v0.24.1) - 2025-07-14

### Other

- *(deps)* bump pinmame-nvram from 0.3.18 to 0.4.1 ([#572](https://github.com/francisdb/vpxtool/pull/572))
- *(deps)* bump toml from 0.8.23 to 0.9.2 ([#573](https://github.com/francisdb/vpxtool/pull/573))
- *(deps)* bump clap from 4.5.40 to 4.5.41 ([#571](https://github.com/francisdb/vpxtool/pull/571))
- *(deps)* bump indicatif from 0.17.11 to 0.18.0 ([#569](https://github.com/francisdb/vpxtool/pull/569))
- *(deps)* bump rust-ini from 0.21.1 to 0.21.2 ([#570](https://github.com/francisdb/vpxtool/pull/570))
- *(deps)* bump Swatinem/rust-cache from 2.7.8 to 2.8.0 ([#567](https://github.com/francisdb/vpxtool/pull/567))
- *(deps)* bump console from 0.15.11 to 0.16.0 ([#566](https://github.com/francisdb/vpxtool/pull/566))
- new clippy rules ([#568](https://github.com/francisdb/vpxtool/pull/568))
- *(deps)* bump pinmame-nvram from 0.3.17 to 0.3.18 ([#565](https://github.com/francisdb/vpxtool/pull/565))
- *(deps)* bump pinmame-nvram from 0.3.16 to 0.3.17 ([#562](https://github.com/francisdb/vpxtool/pull/562))
- *(deps)* bump clap from 4.5.39 to 4.5.40 ([#563](https://github.com/francisdb/vpxtool/pull/563))
- Fix typo in launch_templates doc in README ([#561](https://github.com/francisdb/vpxtool/pull/561))
- *(deps)* bump toml from 0.8.22 to 0.8.23 ([#560](https://github.com/francisdb/vpxtool/pull/560))
- *(deps)* bump clap from 4.5.38 to 4.5.39 ([#559](https://github.com/francisdb/vpxtool/pull/559))
- new clippy rules ([#558](https://github.com/francisdb/vpxtool/pull/558))
- linux developer dependencies
- *(deps)* bump clap from 4.5.37 to 4.5.38 ([#557](https://github.com/francisdb/vpxtool/pull/557))
- *(deps)* bump toml from 0.8.21 to 0.8.22 ([#555](https://github.com/francisdb/vpxtool/pull/555))
- *(deps)* bump chrono from 0.4.40 to 0.4.41 ([#554](https://github.com/francisdb/vpxtool/pull/554))

## [0.24.0](https://github.com/francisdb/vpxtool/compare/v0.23.5...v0.24.0) - 2025-04-29

### Added

- launch templates ([#548](https://github.com/francisdb/vpxtool/pull/548))

### Other

- ignore release step for dependabot
- ignore release step for dependabot
- *(deps)* bump toml from 0.8.20 to 0.8.21 ([#550](https://github.com/francisdb/vpxtool/pull/550))

## [0.23.5](https://github.com/francisdb/vpxtool/compare/v0.23.4...v0.23.5) - 2025-04-21

### Fixed

- --output-dir vbs file name was missing a part ([#547](https://github.com/francisdb/vpxtool/pull/547))

### Other

- *(deps)* bump rand from 0.9.0 to 0.9.1 ([#544](https://github.com/francisdb/vpxtool/pull/544))
- *(deps)* bump pinmame-nvram from 0.3.15 to 0.3.16 ([#543](https://github.com/francisdb/vpxtool/pull/543))
- *(deps)* bump clap from 4.5.36 to 4.5.37 ([#545](https://github.com/francisdb/vpxtool/pull/545))
- *(deps)* bump shellexpand from 3.1.0 to 3.1.1 ([#546](https://github.com/francisdb/vpxtool/pull/546))
- remove bevy linux deps installation
- missed one more deprecated github action step

## [0.23.4](https://github.com/francisdb/vpxtool/compare/v0.23.3...v0.23.4) - 2025-04-15

### Other

- work around github further workflow runs limit

## [0.23.3](https://github.com/francisdb/vpxtool/compare/v0.23.2...v0.23.3) - 2025-04-15

### Other

- switch to dtolnay/rust-toolchain instead of actions-rs/toolchain ([#539](https://github.com/francisdb/vpxtool/pull/539))
- remove docs on gui frontend that was removed

## [0.23.2](https://github.com/francisdb/vpxtool/compare/v0.23.1...v0.23.2) - 2025-04-15

### Other

- do not fail build if git is not available

## [0.23.1](https://github.com/francisdb/vpxtool/compare/v0.23.0...v0.23.1) - 2025-04-15

### Other

- add missing cargo.toml fields required for release
