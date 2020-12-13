cargo-geiger ☢️ 
===============

[![Build Status](https://dev.azure.com/cargo-geiger/cargo-geiger/_apis/build/status/rust-secure-code.cargo-geiger?branchName=master)](https://dev.azure.com/cargo-geiger/cargo-geiger/_build/latest?definitionId=1&branchName=master)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Code Coverage](https://img.shields.io/azure-devops/coverage/cargo-geiger/cargo-geiger/2/master)](https://img.shields.io/azure-devops/coverage/cargo-geiger/cargo-geiger/2/master)
[![crates.io](https://img.shields.io/crates/v/cargo-geiger.svg)](https://crates.io/crates/cargo-geiger)
[![Crates.io](https://img.shields.io/crates/d/cargo-geiger?label=cargo%20installs)](https://crates.io/crates/cargo-geiger)

A program that list statistics related to usage of unsafe Rust code in a Rust
crate and all its dependencies.

This cargo plugin is based on the code from two other projects:
<https://github.com/icefoxen/cargo-osha> and
<https://github.com/sfackler/cargo-tree>.

Installation
------------

Try to find and use a system-wide installed OpenSSL library:
```
cargo install cargo-geiger
```

Or, build and statically link OpenSSL as part of the cargo-geiger executable:
```
cargo install cargo-geiger --features vendored-openssl
```

Usage
-----

1. Navigate to the same directory as the `Cargo.toml` you want to analyze.
2. `cargo geiger`


Output example
--------------

![Example output](https://user-images.githubusercontent.com/3704611/53132247-845f7080-356f-11e9-9c76-a9498d4a744b.png)


Why even care about unsafe Rust usage?
--------------------------------------

When and why to use unsafe Rust is out of scope for this project, it is simply
a tool that provides information to aid auditing and hopefully to guide
dependency selection. It is however the opinion of the author of this project
that __libraries choosing to abstain from unsafe Rust usage when possible should
be promoted__.

This project is an attempt to create pressure against __unnecessary__ usage of
unsafe Rust in public Rust libraries.


Why the name?
-------------

<https://en.wikipedia.org/wiki/Geiger_counter>

Unsafe code and ionizing radiation have something in common, they are both
inevitable in some situations and both should preferably be safely contained!


Known issues
------------

 - Unsafe code inside macros are not detected. Needs macro expansion(?).
 - Unsafe code generated by `build.rs` are probably not detected.
 - More on the github issue tracker.


Roadmap
-------

 - ~~There should be no false negatives. All unsafe code should be
   identified.~~ This is probably too ambitious, but scanning for
   `#![forbid(unsafe_code)]` should be a reliable alternative (implemented since
   0.6.0). Please see the [changelog].
 - An optional whitelist file at the root crate level to specify crates that are
   trusted to use unsafe (should only have an effect if placed in the root
   project).

Libraries
---------

Cargo Geiger exposes three libraries:

 - `cargo-geiger` - Unversioned and highly unstable library exposing the internals of the `cargo-geiger` binary. As such any function contained within this library may be subject to change.
 - `cargo-geiger-serde` - A library containing the serializable report types
 - `geiger` - A library containing some components used by [cargo-geiger] that are decoupled from [cargo]

Changelog
---------

The changelog can be found [here](https://github.com/rust-secure-code/cargo-geiger/blob/master/CHANGELOG.md)

[cargo]: https://crates.io/crates/cargo
[cargo-geiger]: https://crates.io/crates/cargo-geiger
[changelog]: https://github.com/rust-secure-code/cargo-geiger/blob/master/CHANGELOG.md

## Cargo Geiger Safety Report
```

Metric output format: x/y
    x = unsafe code used by the build
    y = total unsafe code found in the crate

Symbols: 
    :) = No `unsafe` usage found, declares #![forbid(unsafe_code)]
    ?  = No `unsafe` usage found, missing #![forbid(unsafe_code)]
    !  = `unsafe` usage found

Functions  Expressions  Impls  Traits  Methods  Dependency

0/0        0/0          0/0    0/0     0/0      :) cargo-geiger 0.10.2
10/10      210/210      0/0    0/0     1/1      !  ├── anyhow 1.0.33
4/4        341/347      0/0    0/0     3/3      !  ├── cargo 0.47.0
10/10      210/210      0/0    0/0     1/1      !  │   ├── anyhow 1.0.33
2/2        45/45        0/0    0/0     0/0      !  │   ├── atty 0.2.14
0/5        12/259       0/0    0/0     2/22     !  │   │   └── libc 0.2.79
0/0        0/0          0/0    0/0     0/0      ?  │   ├── bytesize 1.0.1
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   │       └── serde_derive 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   │           ├── proc-macro2 1.0.24
0/0        0/0          0/0    0/0     0/0      :) │   │           │   └── unicode-xid 0.2.1
0/0        0/0          0/0    0/0     0/0      :) │   │           ├── quote 1.0.7
0/0        0/0          0/0    0/0     0/0      ?  │   │           │   └── proc-macro2 1.0.24
0/0        45/45        3/3    0/0     2/2      !  │   │           └── syn 1.0.53
0/0        0/0          0/0    0/0     0/0      ?  │   │               ├── proc-macro2 1.0.24
0/0        0/0          0/0    0/0     0/0      :) │   │               ├── quote 1.0.7
0/0        0/0          0/0    0/0     0/0      :) │   │               └── unicode-xid 0.2.1
0/0        0/0          0/0    0/0     0/0      ?  │   ├── cargo-platform 0.1.1
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── serde 1.0.117
0/0        1/1          0/0    0/0     0/0      !  │   ├── clap 2.33.3
0/0        23/23        0/0    0/0     0/0      !  │   │   ├── ansi_term 0.11.0
2/2        45/45        0/0    0/0     0/0      !  │   │   ├── atty 0.2.14
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── bitflags 1.2.1
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── strsim 0.8.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── textwrap 0.11.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   └── unicode-width 0.1.8
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── unicode-width 0.1.8
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── vec_map 0.8.2
0/0        0/0          0/0    0/0     0/0      ?  │   │       └── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   ├── crates-io 0.31.1
10/10      210/210      0/0    0/0     1/1      !  │   │   ├── anyhow 1.0.33
4/4        854/855      5/5    0/0     2/2      !  │   │   ├── curl 0.4.34
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   ├── curl-sys 0.4.38+curl-7.73.0
0/5        12/259       0/0    0/0     2/22     !  │   │   │   │   ├── libc 0.2.79
0/0        0/1          0/0    0/0     0/0      ?  │   │   │   │   ├── libnghttp2-sys 0.1.4+1.41.0
0/5        12/259       0/0    0/0     2/22     !  │   │   │   │   │   └── libc 0.2.79
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   │   ├── libz-sys 1.1.2
0/5        12/259       0/0    0/0     2/22     !  │   │   │   │   │   └── libc 0.2.79
39/39      142/142      0/0    0/0     0/0      !  │   │   │   │   └── openssl-sys 0.9.58
0/5        12/259       0/0    0/0     2/22     !  │   │   │   │       └── libc 0.2.79
0/5        12/259       0/0    0/0     2/22     !  │   │   │   ├── libc 0.2.79
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   ├── openssl-probe 0.1.2
39/39      142/142      0/0    0/0     0/0      !  │   │   │   ├── openssl-sys 0.9.58
0/0        585/1063     0/0    0/0     6/10     !  │   │   │   └── socket2 0.3.15
0/0        0/0          0/0    0/0     0/0      ?  │   │   │       ├── cfg-if 0.1.10
0/5        12/259       0/0    0/0     2/22     !  │   │   │       └── libc 0.2.79
0/0        3/3          0/0    0/0     0/0      !  │   │   ├── percent-encoding 2.1.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── serde_derive 1.0.117
0/0        5/5          0/0    0/0     0/0      !  │   │   ├── serde_json 1.0.59
0/0        103/108      1/1    0/0     2/2      !  │   │   │   ├── indexmap 1.6.0
2/2        1006/1098    16/19  0/0     35/39    !  │   │   │   │   ├── hashbrown 0.9.1
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   │   │   └── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   │   └── serde 1.0.117
0/0        1/1          0/0    0/0     0/0      !  │   │   │   ├── itoa 0.4.6
8/12       674/921      0/0    0/0     2/2      !  │   │   │   ├── ryu 1.0.5
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   └── serde 1.0.117
0/0        2/2          0/0    0/0     0/0      !  │   │   └── url 2.1.1
0/0        1/1          0/0    0/0     0/0      !  │   │       ├── idna 0.2.0
0/0        0/0          0/0    0/0     0/0      ?  │   │       │   ├── matches 0.1.8
0/0        0/0          0/0    0/0     0/0      :) │   │       │   ├── unicode-bidi 0.3.4
0/0        0/0          0/0    0/0     0/0      ?  │   │       │   │   ├── matches 0.1.8
0/0        0/0          0/0    0/0     0/0      ?  │   │       │   │   └── serde 1.0.117
0/0        20/20        0/0    0/0     0/0      !  │   │       │   └── unicode-normalization 0.1.13
0/0        0/0          0/0    0/0     0/0      :) │   │       │       └── tinyvec 0.3.4
0/0        0/0          0/0    0/0     0/0      ?  │   │       ├── matches 0.1.8
0/0        3/3          0/0    0/0     0/0      !  │   │       ├── percent-encoding 2.1.0
0/0        0/0          0/0    0/0     0/0      ?  │   │       └── serde 1.0.117
4/4        78/78        14/14  0/0     0/0      !  │   ├── crossbeam-utils 0.7.2
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── cfg-if 0.1.10
0/0        7/7          1/1    0/0     0/0      !  │   │   └── lazy_static 1.4.0
0/0        0/0          0/0    0/0     0/0      ?  │   ├── crypto-hash 0.3.4
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── hex 0.3.2
30/30      5375/5375    29/29  3/3     16/16    !  │   │   └── openssl 0.10.30
0/0        0/0          0/0    0/0     0/0      ?  │   │       ├── bitflags 1.2.1
0/0        0/0          0/0    0/0     0/0      ?  │   │       ├── cfg-if 0.1.10
0/0        0/0          0/0    0/0     0/0      ?  │   │       ├── foreign-types 0.3.2
0/0        0/0          0/0    0/0     0/0      ?  │   │       │   └── foreign-types-shared 0.1.1
0/0        7/7          1/1    0/0     0/0      !  │   │       ├── lazy_static 1.4.0
0/5        12/259       0/0    0/0     2/22     !  │   │       ├── libc 0.2.79
39/39      142/142      0/0    0/0     0/0      !  │   │       └── openssl-sys 0.9.58
4/4        854/855      5/5    0/0     2/2      !  │   ├── curl 0.4.34
0/0        0/0          0/0    0/0     0/0      ?  │   ├── curl-sys 0.4.38+curl-7.73.0
0/0        0/0          0/0    0/0     0/0      ?  │   ├── env_logger 0.7.1
2/2        45/45        0/0    0/0     0/0      !  │   │   ├── atty 0.2.14
0/0        5/5          0/0    0/0     0/0      !  │   │   ├── humantime 1.3.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   └── quick-error 1.2.3
1/1        16/28        0/0    0/0     0/2      !  │   │   ├── log 0.4.11
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   ├── cfg-if 0.1.10
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   └── serde 1.0.117
0/0        34/34        0/1    0/0     2/2      !  │   │   ├── regex 1.4.2
19/19      678/678      0/0    0/0     22/22    !  │   │   │   ├── aho-corasick 0.7.14
26/27      1823/1896    0/0    0/0     0/0      !  │   │   │   │   └── memchr 2.3.3
0/5        12/259       0/0    0/0     2/22     !  │   │   │   │       └── libc 0.2.79
26/27      1823/1896    0/0    0/0     0/0      !  │   │   │   ├── memchr 2.3.3
0/0        0/0          0/0    0/0     0/0      :) │   │   │   ├── regex-syntax 0.6.21
1/1        146/146      2/2    0/0     4/4      !  │   │   │   └── thread_local 1.0.1
0/0        7/7          1/1    0/0     0/0      !  │   │   │       └── lazy_static 1.4.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── termcolor 1.1.0
0/0        21/78        0/0    0/0     0/0      !  │   ├── filetime 0.2.12
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── cfg-if 0.1.10
0/5        12/259       0/0    0/0     2/22     !  │   │   └── libc 0.2.79
4/4        129/129      2/2    0/0     2/2      !  │   ├── flate2 1.0.18
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── cfg-if 0.1.10
5/6        108/156      0/0    0/0     0/0      !  │   │   ├── crc32fast 1.2.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   └── cfg-if 0.1.10
0/5        12/259       0/0    0/0     2/22     !  │   │   ├── libc 0.2.79
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── libz-sys 1.1.2
0/0        0/0          0/0    0/0     0/0      :) │   │   └── miniz_oxide 0.4.3
0/0        0/0          0/0    0/0     0/0      :) │   │       └── adler 0.2.3
6/6        3635/3655    3/3    0/0     80/80    !  │   ├── git2 0.13.12
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── bitflags 1.2.1
0/5        12/259       0/0    0/0     2/22     !  │   │   ├── libc 0.2.79
0/0        18/18        0/0    0/0     0/0      !  │   │   ├── libgit2-sys 0.12.14+1.1.0
0/5        12/259       0/0    0/0     2/22     !  │   │   │   ├── libc 0.2.79
2/2        6/6          0/0    0/0     0/0      !  │   │   │   ├── libssh2-sys 0.2.19
0/5        12/259       0/0    0/0     2/22     !  │   │   │   │   ├── libc 0.2.79
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   │   ├── libz-sys 1.1.2
39/39      142/142      0/0    0/0     0/0      !  │   │   │   │   └── openssl-sys 0.9.58
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   ├── libz-sys 1.1.2
39/39      142/142      0/0    0/0     0/0      !  │   │   │   └── openssl-sys 0.9.58
1/1        16/28        0/0    0/0     0/2      !  │   │   ├── log 0.4.11
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── openssl-probe 0.1.2
39/39      142/142      0/0    0/0     0/0      !  │   │   ├── openssl-sys 0.9.58
0/0        2/2          0/0    0/0     0/0      !  │   │   └── url 2.1.1
1/1        17/19        0/0    0/0     0/0      !  │   ├── git2-curl 0.14.1
4/4        854/855      5/5    0/0     2/2      !  │   │   ├── curl 0.4.34
6/6        3635/3655    3/3    0/0     80/80    !  │   │   ├── git2 0.13.12
1/1        16/28        0/0    0/0     0/2      !  │   │   ├── log 0.4.11
0/0        2/2          0/0    0/0     0/0      !  │   │   └── url 2.1.1
0/0        0/0          0/0    0/0     0/0      ?  │   ├── glob 0.3.0
0/0        0/0          0/0    0/0     0/0      ?  │   ├── hex 0.4.2
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── serde 1.0.117
0/0        0/14         0/0    0/0     0/0      ?  │   ├── home 0.5.3
0/0        0/0          0/0    0/0     0/0      :) │   ├── humantime 2.0.1
0/0        0/0          0/0    0/0     0/0      ?  │   ├── ignore 0.4.16
4/4        78/78        14/14  0/0     0/0      !  │   │   ├── crossbeam-utils 0.7.2
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── globset 0.4.5
19/19      678/678      0/0    0/0     22/22    !  │   │   │   ├── aho-corasick 0.7.14
8/8        365/377      0/0    0/0     0/0      !  │   │   │   ├── bstr 0.2.14
0/0        7/7          1/1    0/0     0/0      !  │   │   │   │   ├── lazy_static 1.4.0
26/27      1823/1896    0/0    0/0     0/0      !  │   │   │   │   ├── memchr 2.3.3
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   │   └── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   ├── fnv 1.0.7
1/1        16/28        0/0    0/0     0/2      !  │   │   │   ├── log 0.4.11
0/0        34/34        0/1    0/0     2/2      !  │   │   │   ├── regex 1.4.2
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   └── serde 1.0.117
0/0        7/7          1/1    0/0     0/0      !  │   │   ├── lazy_static 1.4.0
1/1        16/28        0/0    0/0     0/2      !  │   │   ├── log 0.4.11
26/27      1823/1896    0/0    0/0     0/0      !  │   │   ├── memchr 2.3.3
0/0        34/34        0/1    0/0     2/2      !  │   │   ├── regex 1.4.2
0/0        3/3          0/0    0/0     0/0      !  │   │   ├── same-file 1.0.6
1/1        146/146      2/2    0/0     4/4      !  │   │   ├── thread_local 1.0.1
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── walkdir 2.3.1
0/0        3/3          0/0    0/0     0/0      !  │   │       └── same-file 1.0.6
1/1        122/122      2/2    0/0     4/4      !  │   ├── im-rc 15.0.0
0/0        100/100      0/0    0/0     9/9      !  │   │   ├── bitmaps 2.1.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   └── typenum 1.12.0
0/0        22/22        0/0    0/0     0/0      !  │   │   ├── rand_core 0.5.1
2/4        50/150       1/1    0/0     3/3      !  │   │   │   ├── getrandom 0.1.15
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   │   ├── cfg-if 0.1.10
0/5        12/259       0/0    0/0     2/22     !  │   │   │   │   ├── libc 0.2.79
1/1        16/28        0/0    0/0     0/2      !  │   │   │   │   └── log 0.4.11
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   └── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── rand_xoshiro 0.4.0
0/0        22/22        0/0    0/0     0/0      !  │   │   │   ├── rand_core 0.5.1
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   └── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── serde 1.0.117
0/1        295/615      0/0    0/0     19/38    !  │   │   ├── sized-chunks 0.6.2
0/0        100/100      0/0    0/0     9/9      !  │   │   │   ├── bitmaps 2.1.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   └── typenum 1.12.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── typenum 1.12.0
0/0        188/282      0/2    0/0     4/6      !  │   ├── jobserver 0.1.21
0/5        12/259       0/0    0/0     2/22     !  │   │   └── libc 0.2.79
0/0        7/7          1/1    0/0     0/0      !  │   ├── lazy_static 1.4.0
0/0        43/43        2/2    0/0     0/0      !  │   ├── lazycell 1.3.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── serde 1.0.117
0/5        12/259       0/0    0/0     2/22     !  │   ├── libc 0.2.79
0/0        18/18        0/0    0/0     0/0      !  │   ├── libgit2-sys 0.12.14+1.1.0
1/1        16/28        0/0    0/0     0/2      !  │   ├── log 0.4.11
26/27      1823/1896    0/0    0/0     0/0      !  │   ├── memchr 2.3.3
0/0        72/72        0/0    0/0     0/0      !  │   ├── num_cpus 1.13.0
0/5        12/259       0/0    0/0     2/22     !  │   │   └── libc 0.2.79
0/0        6/6          0/0    0/0     0/0      !  │   ├── opener 0.4.1
30/30      5375/5375    29/29  3/3     16/16    !  │   ├── openssl 0.10.30
0/0        3/3          0/0    0/0     0/0      !  │   ├── percent-encoding 2.1.0
0/0        0/0          0/0    0/0     0/0      ?  │   ├── rustc-workspace-hack 1.0.0
0/0        0/0          0/0    0/0     0/0      ?  │   ├── rustfix 0.5.1
10/10      210/210      0/0    0/0     1/1      !  │   │   ├── anyhow 1.0.33
1/1        16/28        0/0    0/0     0/2      !  │   │   ├── log 0.4.11
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── serde 1.0.117
0/0        5/5          0/0    0/0     0/0      !  │   │   └── serde_json 1.0.59
0/0        3/3          0/0    0/0     0/0      !  │   ├── same-file 1.0.6
0/0        0/4          0/0    0/0     0/0      ?  │   ├── semver 0.10.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── semver-parser 0.7.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   ├── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   ├── serde_ignored 0.1.2
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── serde 1.0.117
0/0        5/5          0/0    0/0     0/0      !  │   ├── serde_json 1.0.59
0/0        0/0          0/0    0/0     0/0      ?  │   ├── shell-escape 0.1.5
0/0        0/0          0/0    0/0     0/0      ?  │   ├── strip-ansi-escapes 0.1.0
0/0        4/5          0/0    0/0     0/0      !  │   │   └── vte 0.3.3
1/1        5/5          0/0    0/0     0/0      !  │   │       └── utf8parse 0.1.1
2/2        52/52        0/0    0/0     0/0      !  │   ├── tar 0.4.30
0/0        21/78        0/0    0/0     0/0      !  │   │   ├── filetime 0.2.12
0/5        12/259       0/0    0/0     2/22     !  │   │   └── libc 0.2.79
0/0        36/82        0/0    0/0     0/0      !  │   ├── tempfile 3.1.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── cfg-if 0.1.10
0/5        12/259       0/0    0/0     2/22     !  │   │   ├── libc 0.2.79
0/0        15/15        0/0    0/0     0/0      !  │   │   ├── rand 0.7.3
2/4        50/150       1/1    0/0     3/3      !  │   │   │   ├── getrandom 0.1.15
0/5        12/259       0/0    0/0     2/22     !  │   │   │   ├── libc 0.2.79
1/1        16/28        0/0    0/0     0/2      !  │   │   │   ├── log 0.4.11
0/0        0/0          0/0    0/0     0/0      ?  │   │   │   ├── rand_chacha 0.2.2
2/2        566/642      0/0    0/0     14/22    !  │   │   │   │   ├── ppv-lite86 0.2.9
0/0        22/22        0/0    0/0     0/0      !  │   │   │   │   └── rand_core 0.5.1
0/0        22/22        0/0    0/0     0/0      !  │   │   │   └── rand_core 0.5.1
0/0        0/79         0/0    0/0     0/0      ?  │   │   └── remove_dir_all 0.5.3
0/0        0/0          0/0    0/0     0/0      ?  │   ├── termcolor 1.1.0
0/0        0/0          0/0    0/0     0/0      ?  │   ├── toml 0.5.7
0/0        103/108      1/1    0/0     2/2      !  │   │   ├── indexmap 1.6.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   ├── unicode-width 0.1.8
0/0        0/0          0/0    0/0     0/0      :) │   ├── unicode-xid 0.2.1
0/0        2/2          0/0    0/0     0/0      !  │   ├── url 2.1.1
0/0        0/0          0/0    0/0     0/0      ?  │   └── walkdir 2.3.1
0/0        0/0          0/0    0/0     0/0      :) ├── cargo-geiger-serde 0.1.0
0/0        0/4          0/0    0/0     0/0      ?  │   ├── semver 0.11.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── semver-parser 0.10.1
2/2        57/57        0/0    0/0     2/2      !  │   │   │   └── pest 2.1.3
0/0        0/0          0/0    0/0     0/0      ?  │   │   │       ├── serde 1.0.117
0/0        5/5          0/0    0/0     0/0      !  │   │   │       ├── serde_json 1.0.59
0/0        0/0          0/0    0/0     0/0      ?  │   │   │       └── ucd-trie 0.1.3
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   ├── serde 1.0.117
0/0        2/2          0/0    0/0     0/0      !  │   └── url 2.1.1
0/0        0/0          0/0    0/0     0/0      ?  ├── cargo-platform 0.1.1
0/0        0/0          0/0    0/0     0/0      ?  ├── cargo_metadata 0.12.0
0/0        0/4          0/0    0/0     0/0      ?  │   ├── semver 0.11.0
0/0        0/0          0/0    0/0     0/0      ?  │   ├── serde 1.0.117
0/0        5/5          0/0    0/0     0/0      !  │   └── serde_json 1.0.59
0/0        13/13        0/0    0/0     0/0      !  ├── colored 2.0.0
2/2        45/45        0/0    0/0     0/0      !  │   ├── atty 0.2.14
0/0        7/7          1/1    0/0     0/0      !  │   └── lazy_static 1.4.0
0/1        148/317      0/0    0/0     0/0      !  ├── console 0.11.3
0/0        7/7          1/1    0/0     0/0      !  │   ├── lazy_static 1.4.0
0/5        12/259       0/0    0/0     2/22     !  │   ├── libc 0.2.79
0/0        34/34        0/1    0/0     2/2      !  │   ├── regex 1.4.2
0/0        5/8          0/0    0/0     0/0      !  │   ├── terminal_size 0.1.13
0/5        12/259       0/0    0/0     2/22     !  │   │   └── libc 0.2.79
2/2        75/75        0/0    0/0     0/0      !  │   ├── termios 0.3.3
0/5        12/259       0/0    0/0     2/22     !  │   │   └── libc 0.2.79
0/0        0/0          0/0    0/0     0/0      ?  │   └── unicode-width 0.1.8
0/0        0/0          0/0    0/0     0/0      ?  ├── env_logger 0.7.1
0/0        0/0          0/0    0/0     0/0      :) ├── geiger 0.4.6
0/0        0/0          0/0    0/0     0/0      :) │   ├── cargo-geiger-serde 0.1.0
0/0        0/0          0/0    0/0     0/0      ?  │   ├── proc-macro2 1.0.24
0/0        45/45        3/3    0/0     2/2      !  │   └── syn 1.0.53
0/0        0/0          0/0    0/0     0/0      ?  ├── krates 0.5.0
0/0        0/0          0/0    0/0     0/0      ?  │   ├── cargo_metadata 0.12.0
0/0        0/0          0/0    0/0     0/0      ?  │   ├── cfg-expr 0.5.0
1/1        402/402      7/7    1/1     13/13    !  │   │   └── smallvec 1.4.2
0/0        0/0          0/0    0/0     0/0      ?  │   │       └── serde 1.0.117
2/2        75/75        4/4    1/1     1/1      !  │   ├── petgraph 0.5.1
0/0        62/62        0/0    0/0     0/0      !  │   │   ├── fixedbitset 0.2.0
0/0        103/108      1/1    0/0     2/2      !  │   │   ├── indexmap 1.6.0
0/0        0/0          0/0    0/0     0/0      ?  │   │   ├── serde 1.0.117
0/0        0/0          0/0    0/0     0/0      ?  │   │   └── serde_derive 1.0.117
0/0        0/4          0/0    0/0     0/0      ?  │   └── semver 0.11.0
2/2        75/75        4/4    1/1     1/1      !  ├── petgraph 0.5.1
0/0        0/0          0/0    0/0     0/0      ?  ├── pico-args 0.3.4
0/0        34/34        0/1    0/0     2/2      !  ├── regex 1.4.2
0/0        0/0          0/0    0/0     0/0      ?  ├── serde 1.0.117
0/0        5/5          0/0    0/0     0/0      !  ├── serde_json 1.0.59
0/0        0/0          0/0    0/0     0/0      ?  ├── strum 0.19.5
0/0        0/0          0/0    0/0     0/0      ?  │   └── strum_macros 0.19.4
0/0        0/0          0/0    0/0     0/0      ?  │       ├── heck 0.3.1
0/0        0/0          0/0    0/0     0/0      ?  │       │   └── unicode-segmentation 1.6.0
0/0        0/0          0/0    0/0     0/0      ?  │       ├── proc-macro2 1.0.24
0/0        0/0          0/0    0/0     0/0      :) │       ├── quote 1.0.7
0/0        45/45        3/3    0/0     2/2      !  │       └── syn 1.0.53
0/0        0/0          0/0    0/0     0/0      ?  ├── strum_macros 0.19.4
0/0        2/2          0/0    0/0     0/0      !  ├── url 2.1.1
0/0        0/0          0/0    0/0     0/0      ?  └── walkdir 2.3.1

191/206    18950/21161  92/98  5/5     250/309

```
