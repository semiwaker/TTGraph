# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.1 (2024-11-15)

### Bug Fixes

 - <csr-id-4a89e9b8ed1e5936b218b0ef1ef2940c5bcfbb27/> Vec<NodeIndex> contains NodeIndex::empty

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 32 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Vec<NodeIndex> contains NodeIndex::empty ([`4a89e9b`](https://github.com/semiwaker/TTGraph/commit/4a89e9b8ed1e5936b218b0ef1ef2940c5bcfbb27))
</details>

## 0.4.0 (2024-10-14)

### New Features

 - <csr-id-2bfd5ca57ae71dc42b709dd44c199ddf8c5c5d42/> cate_arena derive macro runnable
 - <csr-id-51cacf08d0b60e8599abfc62f538e060cf2a9b2d/> improved performance by replacing BTreeMap/Set with OrderMap/Set

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 21 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ttgraph_macros v0.4.0, ttgraph v0.4.0 ([`29288cd`](https://github.com/semiwaker/TTGraph/commit/29288cdc27df10813af869443cb1fbb2a65861a5))
    - BREAKING CHANGE: performance improve & alloc() requires type ([`384c34d`](https://github.com/semiwaker/TTGraph/commit/384c34dfc5fc43385cdd64533e0b02e92b328085))
    - Cate_arena derive macro runnable ([`2bfd5ca`](https://github.com/semiwaker/TTGraph/commit/2bfd5ca57ae71dc42b709dd44c199ddf8c5c5d42))
    - Improved performance by replacing BTreeMap/Set with OrderMap/Set ([`51cacf0`](https://github.com/semiwaker/TTGraph/commit/51cacf08d0b60e8599abfc62f538e060cf2a9b2d))
</details>

## 0.3.1 (2024-09-23)

### New Features

 - <csr-id-e4df1a6b7f0333987dee734dadd5ab5bfc2271aa/> add phantom_group

### Bug Fixes

 - <csr-id-3afb7ffddd4b16ed72abf21cb1b77df2be002c6c/> fix Cargo.toml
 - <csr-id-b6caaadd0209b457cd92d933bcc2e30dcd1ecd41/> fixed the multiple choice problem for bidirectional link

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 2 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ttgraph_macros v0.3.1, ttgraph v0.3.1 ([`e2fc203`](https://github.com/semiwaker/TTGraph/commit/e2fc2033a71b6455aee354602715e83f9c1c8c7d))
    - Fix Cargo.toml ([`3afb7ff`](https://github.com/semiwaker/TTGraph/commit/3afb7ffddd4b16ed72abf21cb1b77df2be002c6c))
    - Fixed the multiple choice problem for bidirectional link ([`b6caaad`](https://github.com/semiwaker/TTGraph/commit/b6caaadd0209b457cd92d933bcc2e30dcd1ecd41))
    - Add phantom_group ([`e4df1a6`](https://github.com/semiwaker/TTGraph/commit/e4df1a6b7f0333987dee734dadd5ab5bfc2271aa))
</details>

## 0.3.0 (2024-09-20)

### Documentation

 - <csr-id-3894c931d8e56ea98d9ff938ba56beaa4e31516d/> write changelog
 - <csr-id-52658bc9f9e78627ae01ae66730d8cb21f7cd3a4/> write changelog
 - <csr-id-bc64c8abad6309cb9f483c8185cd6b6e7b2a00ef/> write changelog
 - <csr-id-0384c53d252dfe83267b66c5f2dde0125227ad87/> write_changelog

### New Features

 - Better grouping, hide generated types ([`f58be19`](https://github.com/semiwaker/TTGraph/commit/f58be195b7f0078fa97d5eade82c43886114aad9))

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 139 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ttgraph_macros v0.3.0, ttgraph v0.3.0 ([`f4a9458`](https://github.com/semiwaker/TTGraph/commit/f4a9458d9b89fbf3f2b91360e6c9022a6a98299f))
    - Write_changelog ([`0384c53`](https://github.com/semiwaker/TTGraph/commit/0384c53d252dfe83267b66c5f2dde0125227ad87))
    - Write changelog ([`bc64c8a`](https://github.com/semiwaker/TTGraph/commit/bc64c8abad6309cb9f483c8185cd6b6e7b2a00ef))
    - Write changelog ([`52658bc`](https://github.com/semiwaker/TTGraph/commit/52658bc9f9e78627ae01ae66730d8cb21f7cd3a4))
    - Write changelog ([`3894c93`](https://github.com/semiwaker/TTGraph/commit/3894c931d8e56ea98d9ff938ba56beaa4e31516d))
    - BREAKING CHANGE: better grouping, hide generated types ([`f58be19`](https://github.com/semiwaker/TTGraph/commit/f58be195b7f0078fa97d5eade82c43886114aad9))
</details>

## 0.2.2 (2024-05-03)

### Documentation

 - <csr-id-9e88418d9b896d07e5e05cb9ff059a7ea9510bdf/> Write changelog

### New Features

 - <csr-id-ac486b1399f808a6c881779a0a64574bbf57e9f9/> Link type check reports more information

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ttgraph_macros v0.2.2, ttgraph v0.2.3 ([`3cdd784`](https://github.com/semiwaker/TTGraph/commit/3cdd784da9f8262733ccb3a08f0613c2656e0758))
    - Write changelog ([`9e88418`](https://github.com/semiwaker/TTGraph/commit/9e88418d9b896d07e5e05cb9ff059a7ea9510bdf))
    - Link type check reports more information ([`ac486b1`](https://github.com/semiwaker/TTGraph/commit/ac486b1399f808a6c881779a0a64574bbf57e9f9))
</details>

## 0.2.1 (2024-05-02)

<csr-id-7696ca920d04b89f9ec112dbf755a1bbd00240e6/>

### Other

 - <csr-id-7696ca920d04b89f9ec112dbf755a1bbd00240e6/> Write changelog

### Documentation

 - <csr-id-78668cf63fdfa2613c1a8ec1cacd4fa8185c8933/> Write changelog

### New Features

 - <csr-id-4e1170114e835e496619d520a86e4aba9eef842d/> Group is supported in link type check

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 9 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ttgraph_macros v0.2.1, ttgraph v0.2.2 ([`61671a3`](https://github.com/semiwaker/TTGraph/commit/61671a340269886c6260b835f5fe610e68872a5e))
    - Write changelog ([`78668cf`](https://github.com/semiwaker/TTGraph/commit/78668cf63fdfa2613c1a8ec1cacd4fa8185c8933))
    - BREAKING CHANGE: Reconstructed the package layout ([`961700c`](https://github.com/semiwaker/TTGraph/commit/961700c7d4c47be2e6be5f63a0549c09f8132389))
    - Write changelog ([`7696ca9`](https://github.com/semiwaker/TTGraph/commit/7696ca920d04b89f9ec112dbf755a1bbd00240e6))
    - Group is supported in link type check ([`4e11701`](https://github.com/semiwaker/TTGraph/commit/4e1170114e835e496619d520a86e4aba9eef842d))
</details>

## 0.2.0 (2024-04-23)

### Added

 - Link check macros

### Changed

 - Impls of TypedNode and NodeEnum

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
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
</details>

## 0.1.0 (2024-04-17)

### Added

 - Basic functionality of TTGraph.

### Changed

 - Changed name from TGraph to TTGraph, as the name tgraph is occupied in crates.io.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release ttgraph_macros v0.1.0, ttgraph v0.1.0 ([`07aa5ac`](https://github.com/semiwaker/TTGraph/commit/07aa5ac027647dbffaaad2dd46f28a42f3eeaac0))
    - Release ttgraph_macros v0.1.0, ttgraph v0.1.0 ([`76089e0`](https://github.com/semiwaker/TTGraph/commit/76089e0ec89fdf3c67d75b6d8ade025d67112303))
    - Release ttgraph_macros v0.1.0, ttgraph v0.1.0 ([`6df6c31`](https://github.com/semiwaker/TTGraph/commit/6df6c3172ba43e4cfc3a922c2721e9934cf28f7b))
    - Add changelog ([`e40361d`](https://github.com/semiwaker/TTGraph/commit/e40361d37ae04c8155f1c9f17f9ae23bb2096f66))
    - Modified for name change ([`29773ce`](https://github.com/semiwaker/TTGraph/commit/29773ce6292b83db04d2b12e863ee87709a560dd))
</details>

