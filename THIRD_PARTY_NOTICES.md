# Third-party dependency inventory (source release)

This inventory is derived from the 90 registry packages in `Cargo.lock` for `mote-agent` 0.13.0. The license expressions below are transcribed from each exact-version crate's `Cargo.toml` in the local Cargo registry; they describe the crates' declared licensing options, **not** a determination of copyright ownership or of which option was exercised. In particular, `OR`, `AND`, `WITH`, and the legacy `MIT/Apache-2.0` spelling are intentionally preserved. The remaining lockfile entry is the root `mote-agent` package, not a third-party crate.

A source-only `cargo package --list --allow-dirty --offline` lists the project files, including `Cargo.lock` and the project's MIT `LICENSE`, but no vendored dependency source or binary. Merely listing a dependency in the lockfile does not redistribute its code. This inventory is informative, rather than a substitute for the license texts, copyright notices, and any upstream `NOTICE` files that must accompany *actually redistributed* third-party works. Include this file in a source release by re-running `cargo package --list --allow-dirty --offline` after adding it.

For a binary or vendored-source release, review its actual target-specific build and payload, then include the applicable upstream license texts and copyright/attribution notices. MIT requires its copyright and permission notice in copies or substantial portions; Unicode-3.0 requires its copyright and permission notice with copies or in associated documentation; Apache-2.0 requires a copy of the license and, where an upstream work has a `NOTICE` file, its applicable attribution notices, as well as the other conditions in section 4.[3][2][1] This inventory **alone is not a complete binary/vendored-source notice bundle**. In particular, `openssl` 0.10.81 is declared **Apache-2.0 only** (not MIT); `unicode-ident` requires Unicode-3.0 **in addition to** a choice between MIT and Apache-2.0; and ICU4X crates below declare Unicode-3.0. The Rust `openssl` crate is distinct from the native OpenSSL library: check separately whether a particular binary distribution includes any native OpenSSL library and its applicable notices. A platform's system TLS library is not automatically bundled with the application.

## Locked registry packages

| Package | Version | Declared license expression |
|---|---|---|
| base64 | 0.22.1 | MIT OR Apache-2.0 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| cc | 1.4.7 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| core-foundation | 0.10.1 | MIT OR Apache-2.0 |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 |
| displaydoc | 0.2.7 | MIT OR Apache-2.0 |
| equivalent | 1.0.2 | Apache-2.0 OR MIT |
| errno | 0.3.14 | MIT OR Apache-2.0 |
| fastrand | 2.5.0 | Apache-2.0 OR MIT |
| find-msvc-tools | 0.1.13 | MIT OR Apache-2.0 |
| foreign-types | 0.3.2 | MIT/Apache-2.0 |
| foreign-types-shared | 0.1.1 | MIT/Apache-2.0 |
| form_urlencoded | 1.2.2 | MIT OR Apache-2.0 |
| getrandom | 0.4.3 | MIT OR Apache-2.0 |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 |
| icu_collections | 2.3.0 | Unicode-3.0 |
| icu_locale_core | 2.3.0 | Unicode-3.0 |
| icu_normalizer | 2.3.0 | Unicode-3.0 |
| icu_normalizer_data | 2.3.0 | Unicode-3.0 |
| icu_properties | 2.3.0 | Unicode-3.0 |
| icu_properties_data | 2.3.0 | Unicode-3.0 |
| icu_provider | 2.3.1 | Unicode-3.0 |
| idna | 1.1.0 | MIT OR Apache-2.0 |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT |
| indexmap | 2.14.2 | Apache-2.0 OR MIT |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| libc | 0.2.189 | MIT OR Apache-2.0 |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| litemap | 0.8.3 | Unicode-3.0 |
| log | 0.4.34 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| native-tls | 0.2.18 | MIT OR Apache-2.0 |
| once_cell | 1.21.4 | MIT OR Apache-2.0 |
| openssl | 0.10.81 | Apache-2.0 |
| openssl-macros | 0.1.1 | MIT/Apache-2.0 |
| openssl-probe | 0.2.1 | MIT OR Apache-2.0 |
| openssl-sys | 0.9.117 | MIT |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 |
| potential_utf | 0.1.6 | Unicode-3.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| rustix | 1.1.4 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 |
| schannel | 0.1.29 | MIT |
| security-framework | 3.7.0 | MIT OR Apache-2.0 |
| security-framework-sys | 2.17.0 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.151 | MIT OR Apache-2.0 |
| serde_yaml | 0.9.34+deprecated | MIT OR Apache-2.0 |
| shell-words | 1.1.1 | MIT/Apache-2.0 |
| shlex | 2.0.1 | MIT OR Apache-2.0 |
| smallvec | 1.16.1 | MIT OR Apache-2.0 |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.6 | MIT OR Apache-2.0 |
| synstructure | 0.14.0 | MIT |
| tempfile | 3.27.0 | MIT OR Apache-2.0 |
| tinystr | 0.8.4 | Unicode-3.0 |
| unicode-ident | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| unsafe-libyaml | 0.2.11 | MIT |
| ureq | 2.12.1 | MIT OR Apache-2.0 |
| url | 2.5.8 | MIT OR Apache-2.0 |
| utf8_iter | 1.0.4 | Apache-2.0 OR MIT |
| vcpkg | 0.2.15 | MIT/Apache-2.0 |
| windows-link | 0.2.1 | MIT OR Apache-2.0 |
| windows-sys | 0.52.0 | MIT OR Apache-2.0 |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 |
| windows-targets | 0.52.6 | MIT OR Apache-2.0 |
| windows_aarch64_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_aarch64_msvc | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_gnu | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_msvc | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_gnu | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 |
| writeable | 0.6.4 | Unicode-3.0 |
| yoke | 0.8.3 | Unicode-3.0 |
| yoke-derive | 0.8.3 | Unicode-3.0 |
| zerofrom | 0.1.8 | Unicode-3.0 |
| zerofrom-derive | 0.1.8 | Unicode-3.0 |
| zerotrie | 0.2.5 | Unicode-3.0 |
| zerovec | 0.11.8 | Unicode-3.0 |
| zerovec-derive | 0.11.6 | Unicode-3.0 |
| zmij | 1.0.23 | MIT |

The lockfile contains packages for multiple targets and build stages; this list does not assert that every package is compiled into every target's executable. For a packaged binary, use the exact upstream license/notice files in each shipped crate version, including any additional third-party notices within a crate, rather than deriving copyright holders from package names.

## Sources

[1] https://www.apache.org/licenses/LICENSE-2.0
[2] https://spdx.org/licenses/Unicode-3.0.html
[3] https://spdx.org/licenses/MIT.html
