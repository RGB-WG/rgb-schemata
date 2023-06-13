# Collection of official RGB schemata

![Build](https://github.com/RGB-WG/rgb-schemata/workflows/Build/badge.svg)
![Tests](https://github.com/RGB-WG/rgb-schemata/workflows/Tests/badge.svg)
![Lints](https://github.com/RGB-WG/rgb-schemata/workflows/Lints/badge.svg)

[![crates.io](https://img.shields.io/crates/v/rgb-schemata)](https://crates.io/crates/rgb-schemata)
[![Docs](https://docs.rs/rgb-schemata/badge.svg)](https://docs.rs/rgb-schemata)
[![Apache-2 licensed](https://img.shields.io/crates/l/rgb-schemata)](./LICENSE)

This repository provides rust source code and compiled versions of RGB
contract schemata recommended for the use by contract developers.

RGB is confidential & scalable client-validated smart contracts for Bitcoin &
Lightning. To learn more about RGB please check [RGB blueprint][Blueprint] and
[RGB FAQ][FAQ] websites.

The development of the project is supported and managed by [LNP/BP Standards
Association][Association].

## Catalog

This repository provides the following RGB schemata:

* __Non-inflatable assets (NIA)__, implementing RGB20 interface.
  This is the simplest form of a fungible asset/token, which doesn't provide
  such features as secondary issue, ability to change asset name and
  parameters, ability to burn or replace the asset.

* __Unique digital asset (UDA)__, implementing RGB21 interface.
  This is the simplest form of an NFT, which has one issuance of a single
  non-fungible and non-fractionable token with a representative attached
  media file and a preview.

* __Collectible fungible assets (CFA)__, implementing RGB25 interface.
  This is the simplest form of collectible fungible assets

## Library

The library can be integrated into other rust projects via `Cargo.toml`
`[dependecies]` section:

```toml
rgb-schemata = "0.10.0"
```

### MSRV

Minimum supported rust compiler version (MSRV): 1.66, rust 2021 edition.

## License

See [LICENCE](LICENSE) file.


[LNPBPs]: https://github.com/LNP-BP/LNPBPs
[Association]: https://lnp-bp.org
[Blueprint]: https://rgb.network
[FAQ]: https://rgbfaq.com
[Foundation]: https://github.com/LNP-BP/client_side_validation
[BP]: https://github.com/BP-WG/bp-core
[RGB Std]: https://github.com/RGB-WG/rgb-std
[RGB Node]: https://github.com/RGB-WG/rgb-node
[Max]: https://github.com/dr-orlovsky
[Todd]: https://petertodd.org/
[Zucco]: https://giacomozucco.com/
