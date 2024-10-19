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

use aluvm::library::{Lib, LibSite};
use amplify::confinement::Confined;
use bp::dbc::Method;
use ifaces::stl::{Amount, Precision, StandardTypes};
use ifaces::{IssuerWrapper, LNPBP_IDENTITY, Rgb20, Rgb20Wrapper};
use rgbstd::containers::ValidContract;
use rgbstd::interface::{
    IfaceClass, IfaceImpl, NamedField, NamedVariant, StateAbi, TxOutpoint, VerNo,
};
use rgbstd::persistence::MemContract;
use rgbstd::schema::{
    GenesisSchema, GlobalStateSchema, Occurrences, OwnedStateSchema, Schema, TransitionSchema,
};
use rgbstd::validation::Scripts;
use rgbstd::{Identity, rgbasm};
use strict_encoding::InvalidRString;
use strict_types::TypeSystem;

use crate::{
    ERRNO_ISSUED_MISMATCH, ERRNO_NON_EQUAL_IN_OUT, GS_ISSUED_SUPPLY, GS_NOMINAL, GS_TERMS,
    OS_ASSET, TS_TRANSFER,
};

pub(crate) fn util_lib() -> Lib {
    rgbasm! {
        // SUBROUTINE Compute sum of inputs
        // Input: a16[16] - state to compute
        // Output: a64[16] - sum
        // Uses:
        // - a16[0]: counter,
        // - a16[10]: zero constant,
        // - a64[0]: extracted amounts
        // - s16[4]: extracted state
        // Fails: on sum overflow or invalid state (should not happen)
        // St0: unmodified if not fails
        put     a16[10],0;              // zero constant
        put     a64[16],0;              // init sum with 0
        cn.i    a16[0],a16[16];         // count state
        dec     a16[0];                 // counter = len - 1
        test;                           // fail if there is no state to load
    /**/ld.i    s16[4],a16[16],a16[0];  // load state
        extr    s16[4],a64[0],a16[10];  // extract 64 bits
        test;                           // fail if state is absent or invalid
        add.uc  a64[16],a64[0];         // add amount to the sum
        test;                           // fail on sum overflow
        dec     a16[0];                 // dec counter
        jif     0x0E;                   // repeat for all assignments
        inv     st0;                    // reset status flag
        ret;                            // finish

        // SUBROUTINE Compute sum of outputs
        // Input: a16[16] - state to compute
        // Output: a64[17] - sum
        // Uses:
        // - a16[0]: counter,
        // - a16[10]: zero constant,
        // - a64[0]: extracted amounts
        // - s16[5]: extracted state
        // Fails: on sum overflow or invalid state (should not happen)
        // St0: unmodified if not fails
        put     a16[10],0;              // zero constant
        put     a64[17],0;              // init sum with 0
        cn.o    a16[0],a16[16];         // count state
        dec     a16[0];                 // counter = len - 1
        test;                           // fail if there is no state to load
    /**/ld.o    s16[5],a16[16],a16[0];  // load state
        extr    s16[5],a64[0],a16[10];  // extract 64 bits
        test;                           // fail if state is absent or invalid
        add.uc  a64[17],a64[0];         // add amount to the sum
        test;                           // fail on sum overflow
        dec     a16[0];                 // dec counter
        jif     0x29;                   // repeat for all assignments
        inv     st0;                    // reset status flag
        ret;                            // finish
    }
}
pub(crate) const FN_UTIL_SUM_INPUTS: u16 = 0;
pub(crate) const FN_UTIL_SUM_OUTPUTS: u16 = 0x22;

pub(crate) fn nia_lib() -> Lib {
    let util = util_lib().id();
    const ISSUED: u16 = GS_ISSUED_SUPPLY.to_u16();
    const DISTRIBUTED: u16 = OS_ASSET.to_u16();
    rgbasm! {
        // SUBROUTINE Genesis validation
        put     a16[0],0;                       // zero constant
        put     a16[15],ISSUED;                 // global state to load
        ld.g    s16[3],a16[15],a16[0];          // load reported issued amount
        put     a16[10],0;                      // zero offset
        extr    s16[3],a64[15],a16[10];         // a64[15] <- GS_ISSUED_SUPPLY
        test;                                   // fail if state is absent or invalid

        put     a16[16],DISTRIBUTED;            // owned state to load
        call    FN_UTIL_SUM_OUTPUTS @ util;     // a64[17] <- sum of OS_ASSET allocations
        put     a8[0],ERRNO_ISSUED_MISMATCH;    // set errno to return if we fail
        eq.n    a64[15],a64[17];                // check if ISSUED =? sum(DISTRIBUTED)
        test;                                   // fail if not
        ret;                                    // complete

        // SUBROUTINE Transfer validation
        put     a16[16],DISTRIBUTED;            // owned state to load
        call    FN_UTIL_SUM_INPUTS @ util;      // a64[16] <- sum of inputs
        call    FN_UTIL_SUM_OUTPUTS @ util;     // a64[17] <- sum of outputs
        put     a8[0],ERRNO_NON_EQUAL_IN_OUT;   // set errno to return if we fail
        eq.n    a64[16],a64[17];                // check if sum(inputs) =? sum(outputs)
        test;                                   // fail if not
        ret;                                    // complete
    }
}
pub(crate) const FN_NIA_GENESIS_OFFSET: u16 = 0;
pub(crate) const FN_NIA_TRANSFER_OFFSET: u16 = 0x24;

fn nia_schema() -> Schema {
    let types = StandardTypes::with(Rgb20::FIXED.stl());

    let alu_lib = nia_lib();
    let alu_id = alu_lib.id();

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
            OS_ASSET => OwnedStateSchema::from(types.get("RGBContract.Amount")),
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
    let lib_id = nia_lib().id();

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
        state_abi: StateAbi {
            reg_input: LibSite::with(0, lib_id),
            reg_output: LibSite::with(0, lib_id),
            calc_output: LibSite::with(0, lib_id),
            calc_change: LibSite::with(0, lib_id),
        },
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
        let util = util_lib();
        let lib = nia_lib();
        Confined::from_checked(bmap! { lib.id() => lib, util.id() => util })
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

    use bp::Txid;
    use bp::seals::txout::{BlindSeal, CloseMethod};
    use ifaces::stl::*;
    use rgbstd::containers::{BuilderSeal, ConsignmentExt};
    use rgbstd::interface::*;
    use rgbstd::{disassemble, *};

    use super::*;

    #[test]
    fn lib_check() {
        let util = util_lib();
        println!("{}", disassemble(&util));
    }

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
        let issued_supply = Amount::from(999u64);
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

        let builder = ContractBuilder::with(
            Identity::default(),
            NonInflatableAsset::FEATURES.iface(),
            NonInflatableAsset::schema(),
            NonInflatableAsset::issue_impl(),
            NonInflatableAsset::types(),
            NonInflatableAsset::scripts(),
        )
        .serialize_global_state("spec", &spec)
        .unwrap()
        .serialize_global_state("terms", &terms)
        .unwrap()
        .serialize_global_state("issuedSupply", &issued_supply)
        .unwrap()
        .serialize_owned_state("assetOwner", BuilderSeal::from(seal), &issued_supply, None)
        .unwrap();

        let contract = builder.issue_contract_det(created_at).unwrap();

        assert_eq!(
            contract.contract_id().to_string(),
            s!("rgb:vGAyeGF9-bPAAV8T-w1V46jM-Iz7TW7K-QzZBzcf-RMuzznw")
        );
    }
}
