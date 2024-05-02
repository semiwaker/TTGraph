# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### New Features

 - <csr-id-4e1170114e835e496619d520a86e4aba9eef842d/> Group is supported in link type check

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 4 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Group is supported in link type check ([`4e11701`](https://github.com/semiwaker/TTGraph/commit/4e1170114e835e496619d520a86e4aba9eef842d))
</details>

## 0.2.1 (2024-04-28)

### Adds

 - `Graph.commit_checked()` and the check module.
 - Implement `IntoIter` for `&Graph`
 - New overloads for `mut_node!` and `update_node!` to support `move ||`

### Documentation

 - <csr-id-b11a80cd342811cf47673c6b0250ce4a7427f87e/> write changelog

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 2 calendar days.
 - 5 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ttgraph v0.2.1 ([`e2501a9`](https://github.com/semiwaker/TTGraph/commit/e2501a96d239ac2dfc7aebbbdc162f874d692380))
    - Write changelog ([`b11a80c`](https://github.com/semiwaker/TTGraph/commit/b11a80cd342811cf47673c6b0250ce4a7427f87e))
    - Adds commit_checked ([`e2a5d35`](https://github.com/semiwaker/TTGraph/commit/e2a5d351795cbde6289024f99828f91c9aa8f04b))
    - Modified mut_node! and update_node! to support move||{} ([`2894b52`](https://github.com/semiwaker/TTGraph/commit/2894b5219e1f33ce4ccd7d4c6ac23e843449aa9f))
</details>

## 0.2.0 (2024-04-23)

### Added

 - Link type check
 - Serialization

### Changed

 - Macro traits TypedGraph and NodeEnum

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 1 calendar day.
 - 5 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ttgraph_macros v0.2.0, ttgraph v0.2.0 ([`c7f4974`](https://github.com/semiwaker/TTGraph/commit/c7f4974049f25e5a83c12a3f5e0eb1db3d76c990))
    - Fixed Change log ([`ac7eb9d`](https://github.com/semiwaker/TTGraph/commit/ac7eb9dcf01ba0441200532233567b01f889a64f))
    - Adjusting changelogs prior to release of ttgraph_macros v0.2.0, ttgraph v0.2.0 ([`6598ad1`](https://github.com/semiwaker/TTGraph/commit/6598ad12b6e0e0ac29d9c78c1ec39b710e6aa02e))
    - Adjusting changelogs prior to release of ttgraph_macros v0.2.0, ttgraph v0.2.0 ([`d0ddff6`](https://github.com/semiwaker/TTGraph/commit/d0ddff647fdc37e7b571d9c9962e5d03034fc1ad))
    - Changelog ([`85488e4`](https://github.com/semiwaker/TTGraph/commit/85488e497d29653dc25f1a6b8fd823d3587aec8d))
    - Add link check ([`bce3e18`](https://github.com/semiwaker/TTGraph/commit/bce3e185e843e9cfafde81770e1195ff360d6f69))
    - Add serialize & deserialize ([`2621e5a`](https://github.com/semiwaker/TTGraph/commit/2621e5abae575f8c35a141c624ee6f9725ac2c70))
</details>

## 0.1.0 (2024-04-17)

### Added

 - Basic functionalities of TTGraph.

### Changed

 - Changed name from TGraph to TTGraph, as the name tgraph is occupied in crates.io.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ttgraph_macros v0.1.0, ttgraph v0.1.0 ([`07aa5ac`](https://github.com/semiwaker/TTGraph/commit/07aa5ac027647dbffaaad2dd46f28a42f3eeaac0))
    - Release ttgraph_macros v0.1.0, ttgraph v0.1.0 ([`76089e0`](https://github.com/semiwaker/TTGraph/commit/76089e0ec89fdf3c67d75b6d8ade025d67112303))
    - Fix cargo.toml ([`2322964`](https://github.com/semiwaker/TTGraph/commit/2322964d067f0b1aa24fae4c158d425c0f8468ce))
    - Fix cargo.toml ([`367ce2f`](https://github.com/semiwaker/TTGraph/commit/367ce2fe6812c7be2b90253916b97eb2535a6a6f))
    - Release ttgraph_macros v0.1.0, ttgraph v0.1.0 ([`6df6c31`](https://github.com/semiwaker/TTGraph/commit/6df6c3172ba43e4cfc3a922c2721e9934cf28f7b))
    - Add changelog ([`e40361d`](https://github.com/semiwaker/TTGraph/commit/e40361d37ae04c8155f1c9f17f9ae23bb2096f66))
    - Modified for name change ([`29773ce`](https://github.com/semiwaker/TTGraph/commit/29773ce6292b83db04d2b12e863ee87709a560dd))
</details>

