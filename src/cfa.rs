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

//! Collectible Fungible Assets (CFA) schema implementing RGB25 fungible assets
//! interface.

use aluvm::library::LibSite;
use amplify::confinement::Confined;
use ifaces::rgb25::Rgb25;
use ifaces::stl::StandardTypes;
use ifaces::{IssuerWrapper, LNPBP_IDENTITY};
use rgbstd::interface::{IfaceClass, IfaceImpl, NamedField, NamedVariant, StateAbi, VerNo};
use rgbstd::schema::{GenesisSchema, GlobalStateSchema, Occurrences, Schema, TransitionSchema};
use rgbstd::validation::Scripts;
use rgbstd::{GlobalStateType, Identity, OwnedStateSchema};
use strict_types::TypeSystem;

use crate::nia::{FN_NIA_GENESIS_OFFSET, FN_NIA_TRANSFER_OFFSET, nia_lib, util_lib};
use crate::{
    ERRNO_ISSUED_MISMATCH, ERRNO_NON_EQUAL_IN_OUT, GS_ISSUED_SUPPLY, GS_TERMS, OS_ASSET,
    TS_TRANSFER,
};

const GS_ART: GlobalStateType = GlobalStateType::with(3000);
const GS_NAME: GlobalStateType = GlobalStateType::with(3001);
const GS_DETAILS: GlobalStateType = GlobalStateType::with(3004);
const GS_PRECISION: GlobalStateType = GlobalStateType::with(3005);

pub fn cfa_schema() -> Schema {
    let types = StandardTypes::with(Rgb25::NONE.stl());

    let nia_id = nia_lib().id();

    Schema {
        ffv: zero!(),
        flags: none!(),
        name: tn!("CollectibleFungibleAsset"),
        timestamp: 1713343888,
        developer: Identity::from(LNPBP_IDENTITY),
        meta_types: none!(),
        global_types: tiny_bmap! {
            GS_ART => GlobalStateSchema::once(types.get("RGBContract.Article")),
            GS_NAME => GlobalStateSchema::once(types.get("RGBContract.Name")),
            GS_DETAILS => GlobalStateSchema::once(types.get("RGBContract.Details")),
            GS_PRECISION => GlobalStateSchema::once(types.get("RGBContract.Precision")),
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
                GS_ART => Occurrences::NoneOrOnce,
                GS_NAME => Occurrences::Once,
                GS_DETAILS => Occurrences::NoneOrOnce,
                GS_PRECISION => Occurrences::Once,
                GS_TERMS => Occurrences::Once,
                GS_ISSUED_SUPPLY => Occurrences::Once,
            },
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::OnceOrMore,
            },
            valencies: none!(),
            validator: Some(LibSite::with(FN_NIA_GENESIS_OFFSET, nia_id)),
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
                validator: Some(LibSite::with(FN_NIA_TRANSFER_OFFSET, nia_id))
            }
        },
        reserved: none!(),
    }
}

pub fn cfa_rgb25() -> IfaceImpl {
    let schema = cfa_schema();
    let lib_id = nia_lib().id();

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: Rgb25::NONE.iface_id(),
        timestamp: 1713343888,
        developer: Identity::from(LNPBP_IDENTITY),
        metadata: none!(),
        global_state: tiny_bset! {
            NamedField::with(GS_ART, fname!("art")),
            NamedField::with(GS_NAME, fname!("name")),
            NamedField::with(GS_DETAILS, fname!("details")),
            NamedField::with(GS_PRECISION, fname!("precision")),
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
pub struct CollectibleFungibleAsset;

impl IssuerWrapper for CollectibleFungibleAsset {
    type IssuingIface = Rgb25;
    const FEATURES: Rgb25 = Rgb25::NONE;

    fn schema() -> Schema { cfa_schema() }
    fn issue_impl() -> IfaceImpl { cfa_rgb25() }

    fn types() -> TypeSystem { StandardTypes::with(Rgb25::NONE.stl()).type_system() }

    fn scripts() -> Scripts {
        let util = util_lib();
        let lib = nia_lib();
        Confined::from_checked(bmap! { lib.id() => lib, util.id() => util })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iimpl_check() {
        let iface = CollectibleFungibleAsset::FEATURES.iface();
        if let Err(err) = cfa_rgb25().check(&iface, &cfa_schema()) {
            for e in err {
                eprintln!("{e}");
            }
            panic!("invalid CFA RGB25 interface implementation");
        }
    }
}
