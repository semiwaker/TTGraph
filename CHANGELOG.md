# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.0 (2024-10-14)

### New Features

 - <csr-id-2bfd5ca57ae71dc42b709dd44c199ddf8c5c5d42/> cate_arena derive macro runnable
 - <csr-id-51cacf08d0b60e8599abfc62f538e060cf2a9b2d/> improved performance by replacing BTreeMap/Set with OrderMap/Set
 - <csr-id-893aad5e2513a964236384487404df240482a3bd/> impl Default for NodeIndex

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 21 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - BREAKING CHANGE: performance improve & alloc() requires type ([`384c34d`](https://github.com/semiwaker/TTGraph/commit/384c34dfc5fc43385cdd64533e0b02e92b328085))
    - Cate_arena derive macro runnable ([`2bfd5ca`](https://github.com/semiwaker/TTGraph/commit/2bfd5ca57ae71dc42b709dd44c199ddf8c5c5d42))
    - Improved performance by replacing BTreeMap/Set with OrderMap/Set ([`51cacf0`](https://github.com/semiwaker/TTGraph/commit/51cacf08d0b60e8599abfc62f538e060cf2a9b2d))
    - Impl Default for NodeIndex ([`893aad5`](https://github.com/semiwaker/TTGraph/commit/893aad5e2513a964236384487404df240482a3bd))
</details>

## 0.3.1 (2024-09-23)

### New Features

 - <csr-id-e4df1a6b7f0333987dee734dadd5ab5bfc2271aa/> add phantom_group

### Bug Fixes

 - <csr-id-b6caaadd0209b457cd92d933bcc2e30dcd1ecd41/> fixed the multiple choice problem for bidirectional link

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fixed the multiple choice problem for bidirectional link ([`b6caaad`](https://github.com/semiwaker/TTGraph/commit/b6caaadd0209b457cd92d933bcc2e30dcd1ecd41))
    - Add phantom_group ([`e4df1a6`](https://github.com/semiwaker/TTGraph/commit/e4df1a6b7f0333987dee734dadd5ab5bfc2271aa))
</details>

## 0.3.0 (2024-09-20)

### New Features

 - Better grouping, hide generated types ([`f58be19`](https://github.com/semiwaker/TTGraph/commit/f58be195b7f0078fa97d5eade82c43886114aad9))

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 139 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - BREAKING CHANGE: better grouping, hide generated types ([`f58be19`](https://github.com/semiwaker/TTGraph/commit/f58be195b7f0078fa97d5eade82c43886114aad9))
    - BREAKING CHANGE: Removed :Sized from NodeEnum and TypedNode; Impl Extend for Transaction; Impl ExactSizedIterator for Graph iterators ([`0ec6db4`](https://github.com/semiwaker/TTGraph/commit/0ec6db49bb3379b0ab82f159edae0abe1066d6c4))
</details>

## 0.2.3 (2024-05-03)

### New Features

 - <csr-id-ac486b1399f808a6c881779a0a64574bbf57e9f9/> Link type check reports more information

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Link type check reports more information ([`ac486b1`](https://github.com/semiwaker/TTGraph/commit/ac486b1399f808a6c881779a0a64574bbf57e9f9))
</details>

## 0.2.2 (2024-05-02)

### New Features

 - <csr-id-4e1170114e835e496619d520a86e4aba9eef842d/> Group is supported in link type check

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 15 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - BREAKING CHANGE: Reconstructed the package layout ([`961700c`](https://github.com/semiwaker/TTGraph/commit/961700c7d4c47be2e6be5f63a0549c09f8132389))
</details>

## 0.2.1 (2024-04-28)

### Adds

 - `Graph.commit_checked()` and the check module.
 - Implement `IntoIter` for `&Graph`
 - New overloads for `mut_node!` and `update_node!` to support `move ||`

### Documentation

 - <csr-id-b11a80cd342811cf47673c6b0250ce4a7427f87e/> write changelog

## 0.2.0 (2024-04-23)

### Added

 - Link type check
 - Serialization

### Changed

 - Macro traits TypedGraph and NodeEnum

## 0.1.0 (2024-04-17)

### Added

 - Basic functionalities of TTGraph.

### Changed

 - Changed name from TGraph to TTGraph, as the name tgraph is occupied in crates.io.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Has Drop verision ([`76b6d69`](https://github.com/semiwaker/TTGraph/commit/76b6d69f32116b5a6d1938b9b97b120ca4955c17))
    - Compilable v1 ([`e64f316`](https://github.com/semiwaker/TTGraph/commit/e64f31638ab689b2d2630fef70f39f821ec8263b))
</details>

