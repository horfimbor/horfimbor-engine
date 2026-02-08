# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.6](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.3.5...horfimbor-eventsource-v0.3.6) - 2026-02-08

### Other

- Rename eventstore to KurrentDb and Update dependencies ([#69](https://github.com/horfimbor/horfimbor-engine/pull/69))
- use let else proposed by clippy ([#66](https://github.com/horfimbor/horfimbor-engine/pull/66))

## [0.3.5](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.3.4...horfimbor-eventsource-v0.3.5) - 2025-07-07

### Other

- switch eventstore to kurrendDb ([#60](https://github.com/horfimbor/horfimbor-engine/pull/60))

## [0.3.4](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.3.3...horfimbor-eventsource-v0.3.4) - 2025-03-30

### Fixed

- fix cache computation ([#59](https://github.com/horfimbor/horfimbor-engine/pull/59))

## [0.3.3](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.3.2...horfimbor-eventsource-v0.3.3) - 2025-03-10

### Other

- add horfimbor-jwt and horfimbor-client-derive ([#57](https://github.com/horfimbor/horfimbor-engine/pull/57))
- edition 2024 ([#55](https://github.com/horfimbor/horfimbor-engine/pull/55))

## [0.3.2](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.3.1...horfimbor-eventsource-v0.3.2) - 2025-02-16

### Other

- update redis to 0.29 (#54)
- allow ModelKey to be the key of a hashmap (#52)

## [0.3.1](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.3.0...horfimbor-eventsource-v0.3.1) - 2025-02-08

### Added

- add helper function on ModelKey (#49)
- replace uuid_v4 by uuid_v7 and allow to create model_key with uuid_v8 (#48)

### Fixed

- display of error was missing (#46)

### Other

- *(deps)* upgrade eventstore (#51)
- *(deps)* upgrade dependencies (#50)

## [0.3.0](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.2.2...horfimbor-eventsource-v0.3.0) - 2024-08-27

### Other
- add doc ([#45](https://github.com/horfimbor/horfimbor-engine/pull/45))
- Add time ([#42](https://github.com/horfimbor/horfimbor-engine/pull/42))
- add cargo machete to CI ([#41](https://github.com/horfimbor/horfimbor-engine/pull/41))
- move error from dto to state ([#40](https://github.com/horfimbor/horfimbor-engine/pull/40))
- move subscription out of dto ([#39](https://github.com/horfimbor/horfimbor-engine/pull/39))
- allow event composition to allow the use of public events ([#37](https://github.com/horfimbor/horfimbor-engine/pull/37))

## [0.2.2](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.2.1...horfimbor-eventsource-v0.2.2) - 2024-08-09

### Other
- upgrade redis ([#35](https://github.com/horfimbor/horfimbor-engine/pull/35))

## [0.2.1](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.2.0...horfimbor-eventsource-v0.2.1) - 2024-04-07

### Fixed
- properly use constant in macro ([#33](https://github.com/horfimbor/horfimbor-engine/pull/33))

## [0.2.0](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.1.2...horfimbor-eventsource-v0.2.0) - 2024-03-17

### Added
- add a stronger clippy ([#31](https://github.com/horfimbor/horfimbor-engine/pull/31))

## [0.1.2](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.1.1...horfimbor-eventsource-v0.1.2) - 2024-03-06

### Other
- release ([#27](https://github.com/horfimbor/horfimbor-engine/pull/27))

## [0.1.1](https://github.com/horfimbor/horfimbor-engine/compare/horfimbor-eventsource-v0.1.0...horfimbor-eventsource-v0.1.1) - 2024-03-06

### Other
- release ([#24](https://github.com/horfimbor/horfimbor-engine/pull/24))

## [0.1.0](https://github.com/horfimbor/horfimbor-engine/releases/tag/horfimbor-eventsource-v0.1.0) - 2024-03-06

### Other
- Fix license ([#26](https://github.com/horfimbor/horfimbor-engine/pull/26))
- move into a workspace ([#25](https://github.com/horfimbor/horfimbor-engine/pull/25))
