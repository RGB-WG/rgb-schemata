// RGB schemata by LNP/BP Standards Association
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2023-2024 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2023-2024 LNP/BP Standards Association. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Non-Inflatable Assets (NIA) schema implementing RGB20 fungible assets
//! interface.

use aluvm::isa::opcodes::INSTR_PUTA;
use aluvm::isa::Instr;
use aluvm::library::{Lib, LibSite};
use amplify::confinement::Confined;
use bp::dbc::Method;
use ifaces::{IssuerWrapper, Rgb20, Rgb20Wrapper, LNPBP_IDENTITY};
use rgbstd::containers::ValidContract;
use rgbstd::interface::{IfaceClass, IfaceImpl, NamedField, NamedVariant, TxOutpoint, VerNo};
use rgbstd::persistence::MemContract;
use rgbstd::schema::{
    FungibleType, GenesisSchema, GlobalStateSchema, Occurrences, OwnedStateSchema, Schema,
    TransitionSchema,
};
use rgbstd::stl::StandardTypes;
use rgbstd::validation::Scripts;
use rgbstd::vm::opcodes::INSTR_PCVS;
use rgbstd::vm::RgbIsa;
use rgbstd::{rgbasm, Amount, Identity, Precision};
use strict_encoding::InvalidRString;
use strict_types::TypeSystem;

use crate::{
    ERRNO_ISSUED_MISMATCH, ERRNO_NON_EQUAL_IN_OUT, GS_ISSUED_SUPPLY, GS_NOMINAL, GS_TERMS,
    OS_ASSET, TS_TRANSFER,
};

pub(crate) fn nia_lib() -> Lib {
    let code = rgbasm! {
        // SUBROUTINE Transfer validation
        // Set errno
        put     a8[0],ERRNO_NON_EQUAL_IN_OUT;
        // Checking that the sum of pedersen commitments in inputs is equal to the sum in outputs.
        pcvs    OS_ASSET;
        test;
        ret;

        // SUBROUTINE Genesis validation
        // Checking pedersen commitments against reported amount of issued assets present in the
        // global state.
        put     a8[0],ERRNO_ISSUED_MISMATCH;
        put     a8[1],0;
        put     a16[0],0;
        // Read global state into s16[0]
        ldg     GS_ISSUED_SUPPLY,a8[1],s16[0];
        // Extract 64 bits from the beginning of s16[0] into a64[1]
        // NB: if the global state is invalid, we will fail here and fail the validation
        extr    s16[0],a64[0],a16[0];
        // verify sum of pedersen commitments for assignments against a64[0] value
        pcas    OS_ASSET;
        test;
        ret;
    };
    Lib::assemble::<Instr<RgbIsa<MemContract>>>(&code).expect("wrong non-inflatable asset script")
}
pub(crate) const FN_NIA_GENESIS_OFFSET: u16 = 4 + 3 + 2;
pub(crate) const FN_NIA_TRANSFER_OFFSET: u16 = 0;

fn nia_schema() -> Schema {
    let types = StandardTypes::with(Rgb20::FIXED.stl());

    let alu_lib = nia_lib();
    let alu_id = alu_lib.id();
    assert_eq!(alu_lib.code.as_ref()[FN_NIA_TRANSFER_OFFSET as usize + 4], INSTR_PCVS);
    assert_eq!(alu_lib.code.as_ref()[FN_NIA_GENESIS_OFFSET as usize], INSTR_PUTA);
    assert_eq!(alu_lib.code.as_ref()[FN_NIA_GENESIS_OFFSET as usize + 4], INSTR_PUTA);
    assert_eq!(alu_lib.code.as_ref()[FN_NIA_GENESIS_OFFSET as usize + 8], INSTR_PUTA);

    Schema {
        ffv: zero!(),
        flags: none!(),
        name: tn!("NonInflatableAsset"),
        timestamp: 1713343888,
        developer: Identity::from(LNPBP_IDENTITY),
        meta_types: none!(),
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalStateSchema::once(types.get("RGBContract.AssetSpec")),
            GS_TERMS => GlobalStateSchema::once(types.get("RGBContract.ContractTerms")),
            GS_ISSUED_SUPPLY => GlobalStateSchema::once(types.get("RGBContract.Amount")),
        },
        owned_types: tiny_bmap! {
            OS_ASSET => OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: none!(),
            globals: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_TERMS => Occurrences::Once,
                GS_ISSUED_SUPPLY => Occurrences::Once,
            },
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::OnceOrMore,
            },
            valencies: none!(),
            validator: Some(LibSite::with(FN_NIA_GENESIS_OFFSET, alu_id)),
        },
        extensions: none!(),
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionSchema {
            metadata: none!(),
                globals: none!(),
                inputs: tiny_bmap! {
                    OS_ASSET => Occurrences::OnceOrMore
                },
                assignments: tiny_bmap! {
                    OS_ASSET => Occurrences::OnceOrMore
                },
                valencies: none!(),
                validator: Some(LibSite::with(FN_NIA_TRANSFER_OFFSET, alu_id))
            }
        },
        reserved: none!(),
    }
}

fn nia_rgb20() -> IfaceImpl {
    let schema = nia_schema();
    let iface = Rgb20::FIXED;

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        timestamp: 1713343888,
        developer: Identity::from(LNPBP_IDENTITY),
        metadata: none!(),
        global_state: tiny_bset! {
            NamedField::with(GS_NOMINAL, fname!("spec")),
            NamedField::with(GS_TERMS, fname!("terms")),
            NamedField::with(GS_ISSUED_SUPPLY, fname!("issuedSupply")),
        },
        assignments: tiny_bset! {
            NamedField::with(OS_ASSET, fname!("assetOwner")),
        },
        valencies: none!(),
        transitions: tiny_bset! {
            NamedField::with(TS_TRANSFER, fname!("transfer")),
        },
        extensions: none!(),
        errors: tiny_bset![
            NamedVariant::with(ERRNO_ISSUED_MISMATCH, vname!("issuedMismatch")),
            NamedVariant::with(ERRNO_NON_EQUAL_IN_OUT, vname!("nonEqualAmounts")),
        ],
    }
}

#[derive(Default)]
pub struct NonInflatableAsset;

impl IssuerWrapper for NonInflatableAsset {
    const FEATURES: Rgb20 = Rgb20::FIXED;
    type IssuingIface = Rgb20;

    fn schema() -> Schema { nia_schema() }
    fn issue_impl() -> IfaceImpl { nia_rgb20() }

    fn types() -> TypeSystem { StandardTypes::with(Self::FEATURES.stl()).type_system() }

    fn scripts() -> Scripts {
        let lib = nia_lib();
        Confined::from_checked(bmap! { lib.id() => lib })
    }
}

impl NonInflatableAsset {
    pub fn testnet(
        issuer: &str,
        ticker: &str,
        name: &str,
        details: Option<&str>,
        precision: Precision,
        allocations: impl IntoIterator<Item = (Method, impl TxOutpoint, impl Into<Amount>)>,
    ) -> Result<ValidContract, InvalidRString> {
        let mut issuer =
            Rgb20Wrapper::<MemContract>::testnet::<Self>(issuer, ticker, name, details, precision)?;
        for (method, beneficiary, amount) in allocations {
            issuer = issuer
                .allocate(method, beneficiary, amount)
                .expect("invalid contract data");
        }
        Ok(issuer.issue_contract().expect("invalid contract data"))
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bp::seals::txout::{BlindSeal, CloseMethod};
    use bp::Txid;
    use chrono::DateTime;
    use rgbstd::containers::{BuilderSeal, ConsignmentExt};
    use rgbstd::interface::*;
    use rgbstd::invoice::Precision;
    use rgbstd::stl::*;
    use rgbstd::*;

    use super::*;

    #[test]
    fn iimpl_check() {
        let iface = NonInflatableAsset::FEATURES.iface();
        if let Err(err) = nia_rgb20().check(&iface, &nia_schema()) {
            for e in err {
                eprintln!("{e}");
            }
            panic!("invalid NIA RGB20 interface implementation");
        }
    }

    #[test]
    fn deterministic_contract_id() {
        let created_at = 1713261744;
        let terms = ContractTerms {
            text: RicardianContract::default(),
            media: None,
        };
        let spec = AssetSpec {
            ticker: Ticker::from("TICKER"),
            name: Name::from("NAME"),
            details: None,
            precision: Precision::try_from(2).unwrap(),
        };
        let issued_supply = 999u64;
        let seal: XChain<BlindSeal<Txid>> = XChain::with(
            Layer1::Bitcoin,
            GenesisSeal::from(BlindSeal::with_blinding(
                CloseMethod::OpretFirst,
                Txid::from_str("8d54c98d4c29a1ec4fd90635f543f0f7a871a78eb6a6e706342f831d92e3ba19")
                    .unwrap(),
                0,
                654321,
            )),
        );
        let asset_tag = AssetTag::new_deterministic(
            "contract_domain",
            AssignmentType::with(0),
            DateTime::from_timestamp(created_at, 0).unwrap(),
            123456,
        );

        let builder = ContractBuilder::deterministic(
            Identity::default(),
            NonInflatableAsset::FEATURES.iface(),
            NonInflatableAsset::schema(),
            NonInflatableAsset::issue_impl(),
            NonInflatableAsset::types(),
            NonInflatableAsset::scripts(),
        )
        .add_global_state("spec", spec)
        .unwrap()
        .add_global_state("terms", terms)
        .unwrap()
        .add_global_state("issuedSupply", Amount::from(issued_supply))
        .unwrap()
        .add_asset_tag("assetOwner", asset_tag)
        .unwrap()
        .add_fungible_state_det(
            "assetOwner",
            BuilderSeal::from(seal),
            issued_supply,
            BlindingFactor::from_str(
                "a3401bcceb26201b55978ff705fecf7d8a0a03598ebeccf2a947030b91a0ff53",
            )
            .unwrap(),
        )
        .unwrap();

        let contract = builder.issue_contract_det(created_at).unwrap();

        assert_eq!(
            contract.contract_id().to_string(),
            s!("rgb:pOIzGFyQ-mA!yQq2-QH8vB5!-5fAplY!-x2lW!vz-JHDbYPg")
        );
    }
}
