# Changelog - [get-size2](https://github.com/bircni/get-size2)

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

## [0.9.0](https://github.com/bircni/get-size2/compare/0.8.0..0.9.0) - 2026-05-15

### Features

- Add parking_lot feature with GetSize + GetSizeTracker impls (#46) - ([18b15c8](https://github.com/bircni/get-size2/commit/18b15c89b3ab662a7ca3c5c4b9df82dc41e1093a)) - Al Johri
- Add half feature with GetSize impls for f16 and bf16 (#47) - ([725777b](https://github.com/bircni/get-size2/commit/725777b9049dd50b6791c6e45add1ecd2ad282e7)) - Al Johri
- Add dashmap feature with GetSize impls for DashMap and DashSet (#48) - ([faf1b09](https://github.com/bircni/get-size2/commit/faf1b09dfb4ae16a296f89ead81fe82ba3495036)) - Al Johri

## [0.8.0](https://github.com/bircni/get-size2/compare/0.7.4..0.8.0) - 2026-04-20

### Bug Fixes

- **(changelog)** add space after breaking change indicator in commit message formatting - ([dd2c41f](https://github.com/bircni/get-size2/commit/dd2c41f8ffef291f280b437d311aaae19e915ba5)) - Nicolas

### Features

- **(dependencies)** update indexmap to 2.13 and ordermap to 1.1 - ([af4b9dd](https://github.com/bircni/get-size2/commit/af4b9ddb50631feceecc18e3cca2630de6a4e8af)) - Nicolas
- **(dependencies)** Update hashbrown, indexmap & ordermap - ([b152bfd](https://github.com/bircni/get-size2/commit/b152bfddd05a44c08620d02d3cd47e94c7ee0cee)) - Nicolas

### Tests

- Enhance test coverage and layout - ([f676529](https://github.com/bircni/get-size2/commit/f6765296b39d3b419ee524d61365b2e744bae9cb)) - Nicolas

## [0.7.4](https://github.com/bircni/get-size2/compare/0.7.3..0.7.4) - 2026-01-25

### Bug Fixes

- **(derive)** improve error handling - ([ef02aef](https://github.com/bircni/get-size2/commit/ef02aef31c71cfccf9d024ceb61706d4980aabe0)) - Nicolas
- **(sync)** handle poisoned locks - ([27af5ea](https://github.com/bircni/get-size2/commit/27af5ea22543b0dbf2d8b8c0ce04d46645fbc364)) - Nicolas

### Refactoring

- **(get-size2)** organize feature impls into separate files - ([d051b56](https://github.com/bircni/get-size2/commit/d051b569de309de6c4f91964e5f6802b48a4a0d7)) - Nicolas
- Refactor GetSize implementation by modularizing code into separate files - ([6480486](https://github.com/bircni/get-size2/commit/6480486eff5db08d1cfa80548bdb40709bd8ff07)) - Nicolas

## [0.7.3](https://github.com/bircni/get-size2/compare/0.7.2..0.7.3) - 2025-12-04

### Features

- Implement GetSize for ordermap - ([662fb2e](https://github.com/bircni/get-size2/commit/662fb2e70aed767aba2a49d2bab0bedfc36ff930)) - Jack O'Connor

## [0.7.2](https://github.com/bircni/get-size2/compare/0.7.1..0.7.2) - 2025-11-13

### Documentation

- Update docs with correct links - ([b234d70](https://github.com/bircni/get-size2/commit/b234d70ece314ae80cc993f077f15a8fc0dd583d)) - Nicolas

## [0.7.1](https://github.com/bircni/get-size2/compare/0.7.0..0.7.1) - 2025-10-26

### Bug Fixes

- add missing LICENSE file in the published get-size2 crate and update authors (#36) - ([b6f2970](https://github.com/bircni/get-size2/commit/b6f29700a268daa9e3fd1954bfe04906e43d90f7)) - Ben Beasley

### Features

- implement GetSize for RefCell<T> - ([8ee6af7](https://github.com/bircni/get-size2/commit/8ee6af749625263b77c4c0a1c6106f3109a49367)) - Nicolas

## [0.7.0](https://github.com/bircni/get-size2/compare/0.6.3..0.7.0) - 2025-09-20

### Features

-  [**breaking**] Keep size tracker alive through recursive calls to `get_heap_size_with_tracker` (#34) - ([39a9baf](https://github.com/bircni/get-size2/commit/39a9bafa76f495bf526bbc39341843bb3e03c1ec)) - Ibraheem Ahmed

### Miscellaneous Chores

- set correct version for get-size-derive - ([d3abc1d](https://github.com/bircni/get-size2/commit/d3abc1d9e9f91c2d41f7f2faf0505188909a0721)) - Nicolas

## [0.6.3](https://github.com/bircni/get-size2/compare/0.6.2..0.6.3) - 2025-09-19

### Features

- update `hashbrown` to 0.16.0 (#35) - ([551a0bb](https://github.com/bircni/get-size2/commit/551a0bb2858fd984b2414a05bb743cb5934ced88)) - Ibraheem Ahmed

## [0.6.2](https://github.com/bircni/get-size2/compare/0.6.1..0.6.2) - 2025-08-06

### Bug Fixes

- remove `'static` requirement for tracked objects (#33) - ([209bb8c](https://github.com/bircni/get-size2/commit/209bb8c1c6672df0f55d8c816085ef5b771ba578)) - Ibraheem Ahmed

## [0.6.1](https://github.com/bircni/get-size2/compare/0.6.0..0.6.1) - 2025-07-26

### Bug Fixes

- update Cow implementation to support unsized types and add tests for heap size calculation (#31) - ([45e957d](https://github.com/bircni/get-size2/commit/45e957d1ee337731840004d0cca4d67744e20cf1)) - Zhu He

### Features

- implement GetSize for Rc and Arc slices with corresponding tests (#32) - ([e32d05f](https://github.com/bircni/get-size2/commit/e32d05fe781a3b96973f0ef479b47d8b563be4fe)) - Zhu He

## [0.6.0](https://github.com/bircni/get-size2/compare/0.5.2..0.6.0) - 2025-07-23

### Bug Fixes

- update version for get-size-derive - ([db495ea](https://github.com/bircni/get-size2/commit/db495eab7c650ecb8b9c0827a78f05c2e02c9540)) - Nicolas
- heap size calculation for spilled `SmallVec` (#28) - ([e6b5381](https://github.com/bircni/get-size2/commit/e6b5381e42407e4d7268f454cdee71036431e4be)) - Micha Reiser
- heap size calculation for spilled `CompactStr` (#30) - ([c730c67](https://github.com/bircni/get-size2/commit/c730c67281371b73386737f78311c593a2752b90)) - Micha Reiser
- simplify iteration over elements in heap size calculation - ([5e3feda](https://github.com/bircni/get-size2/commit/5e3fedaf5de351d50ce81d21e8c4fd7d63af5e3e)) - Nicolas

### Features

- Add `ThinVec` support (#29) - ([c46839c](https://github.com/bircni/get-size2/commit/c46839c990d4318712dacfb6e3d3863a230e2d23)) - Micha Reiser

## [0.5.2](https://github.com/bircni/get-size2/compare/0.5.1..0.5.2) - 2025-07-09

### Features

- Optionally implement GetSize for indexmap (#26) - ([d6e3310](https://github.com/bircni/get-size2/commit/d6e3310160a498461ff60b9581f2e4d951cd33bd)) - Brian Janssen

### Lint

- fix new lints in rust 1.88 - ([7292f5e](https://github.com/bircni/get-size2/commit/7292f5e1968209e2091213dc6ebf5b9e2e226058)) - Nicolas

## [0.5.1](https://github.com/bircni/get-size2/compare/0.5.0..0.5.1) - 2025-06-25

### Bug Fixes

- correctly determine size for enums (#24) - ([3c5bd18](https://github.com/bircni/get-size2/commit/3c5bd18cac7d521a7292db65f400d666739b6008)) - Nicolas

### Miscellaneous Chores

- add top-level `heap_size` function (#25) - ([f3b5e6e](https://github.com/bircni/get-size2/commit/f3b5e6e38cc3bc57980a6110868e02f1de4a7982)) - Ibraheem Ahmed

### Build

- update to newer cargo-verset to set dependency version automatically - ([b1154e4](https://github.com/bircni/get-size2/commit/b1154e457291a7dedb16dc5587cae3efea537411)) - Nicolas

## [0.5.0](https://github.com/bircni/get-size2/compare/0.4.1..0.5.0) - 2025-06-25

### Bug Fixes

- account for padding in `HashMap` allocation size (#23) - ([492b9d8](https://github.com/bircni/get-size2/commit/492b9d8982e807b4d3736012cc0f3e05289425af)) - Ibraheem Ahmed

### Features

-  [**breaking**] Impl for all `Range` types, while accounting for possible heap-allocations of indices. (#16) - ([90d354b](https://github.com/bircni/get-size2/commit/90d354b3799ffe264e127c9a9daad76a3f2dedad)) - Jasper
- Add `smallvec` feature (#20) - ([e18b27e](https://github.com/bircni/get-size2/commit/e18b27ef9e4bdd5041b4007d8d9d0bc952cf2a47)) - Ibraheem Ahmed
- Add `hashbrown` feature (#21) - ([02b5cfd](https://github.com/bircni/get-size2/commit/02b5cfdd37ac9509b56b7945a647f102973c29ba)) - Ibraheem Ahmed
- Add `compact-str` feature (#22) - ([97b6303](https://github.com/bircni/get-size2/commit/97b6303878f1fa6f3e40b2ab0fe6a95d90b51e3f)) - Ibraheem Ahmed
- Implement `GetSize` for `OnceLock` (#19) - ([6480592](https://github.com/bircni/get-size2/commit/64805922bd9f7c86905225f7edcef505eb773593)) - Ibraheem Ahmed
- implement `GetSize` for `Box<str>` (#18) - ([8576eb4](https://github.com/bircni/get-size2/commit/8576eb4edf33574f6588dc8f875c406caa7da7d7)) - Ibraheem Ahmed

## [0.4.0](https://github.com/bircni/get-size2/compare/0.3.0..0.4.0) - 2025-06-17

### Features

- Add default impl for Range<I> (#15) - ([ca4ce14](https://github.com/bircni/get-size2/commit/ca4ce143dc506850e1e5f327c42621aeecb3a086)) - Jasper
- Generalize impl for Hash{Set,Map} for all Hashers (#14) - ([dc825d1](https://github.com/bircni/get-size2/commit/dc825d1923ccb202703efd02413619e01585f1fc)) - Jasper

### Build

- prepare for rust 1.87 (#13) - ([2c83579](https://github.com/bircni/get-size2/commit/2c83579b91bf4281db9799c4f02d04bdd92993b3)) - Nicolas

## [0.3.0](https://github.com/bircni/get-size2/compare/0.2.0..0.3.0) - 2025-04-18

### Features

- add release scripts - ([8b85a7f](https://github.com/bircni/get-size2/commit/8b85a7fde760f4455a4fabe4e8eed6935d0ee179)) - Nicolas
- Optionally implement GetSize for bytes::Bytes (#12) - ([44b5c60](https://github.com/bircni/get-size2/commit/44b5c609f1dc6c0e677faf08ee71ba6bd3a7e484)) - Joe Roback

## [0.1.4](https://github.com/bircni/get-size2/compare/0.1.3..0.1.4) - 2025-03-18

### Features

- **(chrono)** Optionally implement GetSize for chrono and chrono-tz (#9) - ([309aab0](https://github.com/bircni/get-size2/commit/309aab024f4f330c507f8e346593c7aedfa14166)) - Brian Janssen
- **(url)** Optionally implement GetSize for url (#8) - ([522be10](https://github.com/bircni/get-size2/commit/522be106862d41cc9a7b8421525015ae35c6cccc)) - Brian Janssen

### Miscellaneous Chores

- **(ci)** refactor ci workflow (#10) - ([f425714](https://github.com/bircni/get-size2/commit/f4257143955d79aa954e752978f592bfa92b8c18)) - Nicolas

## [0.1.2](https://github.com/bircni/get-size2/compare/0.1.1..0.1.2) - 2024-09-14

### Feature

- Remove need of use get_size::GetSize for #[derive(get_size::GetSize)] (#3) - ([067e8e3](https://github.com/bircni/get-size2/commit/067e8e37fc0071497f90e51726f1c3819f11246d)) - Nicolas

## [0.1.4] - 2023-06-23
