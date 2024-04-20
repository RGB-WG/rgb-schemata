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
use ifaces::rgb25::Rgb25;
use ifaces::{rgb25, IssuerWrapper, LNPBP_IDENTITY};
use rgbstd::interface::{IfaceClass, IfaceImpl, NamedField, VerNo};
use rgbstd::schema::{
    FungibleType, GenesisSchema, GlobalStateSchema, Occurrences, Schema, TransitionSchema,
};
use rgbstd::stl::StandardTypes;
use rgbstd::validation::Scripts;
use rgbstd::{GlobalStateType, Identity, OwnedStateSchema};
use strict_types::TypeSystem;

use crate::nia::{nia_lib, FN_NIA_GENESIS_OFFSET, FN_NIA_TRANSFER_OFFSET};
use crate::{GS_ISSUED_SUPPLY, GS_TERMS, OS_ASSET, TS_TRANSFER};

const GS_NAME: GlobalStateType = GlobalStateType::with(2000);
const GS_DETAILS: GlobalStateType = GlobalStateType::with(2004);
const GS_PRECISION: GlobalStateType = GlobalStateType::with(2005);

pub fn cfa_schema() -> Schema {
    let types = StandardTypes::with(Rgb25::stl());

    let alu_lib = nia_lib();
    let alu_id = alu_lib.id();

    Schema {
        ffv: zero!(),
        flags: none!(),
        name: tn!("CollectibleFungibleAsset"),
        timestamp: 1713343888,
        developer: Identity::from(LNPBP_IDENTITY),
        meta_types: none!(),
        global_types: tiny_bmap! {
            GS_NAME => GlobalStateSchema::once(types.get("RGBContract.Name")),
            GS_DETAILS => GlobalStateSchema::once(types.get("RGBContract.Details")),
            GS_PRECISION => GlobalStateSchema::once(types.get("RGBContract.Precision")),
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

pub fn cfa_rgb25() -> IfaceImpl {
    let schema = cfa_schema();
    let iface = Rgb25::iface(rgb25::Features::NONE);

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        timestamp: 1713343888,
        developer: Identity::from(LNPBP_IDENTITY),
        metadata: none!(),
        global_state: tiny_bset! {
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
        errors: none!(), // TODO: Encode errors
    }
}

pub struct CollectibleFungibleAsset;

impl IssuerWrapper for CollectibleFungibleAsset {
    const FEATURES: rgb25::Features = rgb25::Features::NONE;
    type IssuingIface = Rgb25;

    fn schema() -> Schema { cfa_schema() }
    fn issue_impl() -> IfaceImpl { cfa_rgb25() }

    fn types() -> TypeSystem { StandardTypes::with(Rgb25::stl()).type_system() }

    fn scripts() -> Scripts {
        let lib = nia_lib();
        confined_bmap! { lib.id() => lib }
    }
}
