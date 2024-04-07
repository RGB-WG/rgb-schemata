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

use aluvm::library::{Lib, LibSite};
use chrono::Utc;
use rgbstd::interface::{rgb25, IfaceClass, IfaceImpl, NamedField, Rgb25, VerNo};
use rgbstd::schema::{
    FungibleType, GenesisSchema, GlobalStateSchema, Occurrences, Schema, Script, StateSchema,
    TransitionSchema,
};
use rgbstd::stl::StandardTypes;
use rgbstd::vm::{AluLib, AluScript, ContractOp, EntryPoint, RgbIsa};
use rgbstd::{GlobalStateType, Types};
use strict_types::{SemId, Ty};

use crate::{GS_ISSUED_SUPPLY, GS_TERMS, OS_ASSET, TS_TRANSFER};

const GS_NAME: GlobalStateType = GlobalStateType::with(2000);
const GS_DETAILS: GlobalStateType = GlobalStateType::with(2004);
const GS_PRECISION: GlobalStateType = GlobalStateType::with(2005);

pub fn cfa_schema() -> Schema {
    let types = StandardTypes::with(Rgb25::stl());

    let code = [RgbIsa::Contract(ContractOp::PcVs(OS_ASSET))];
    let alu_lib = AluLib(Lib::assemble(&code).unwrap());
    let alu_id = alu_lib.id();

    Schema {
        ffv: zero!(),
        flags: none!(),
        types: Types::Strict(types.type_system()),
        global_types: tiny_bmap! {
            GS_NAME => GlobalStateSchema::once(types.get("RGBContract.Name")),
            GS_DETAILS => GlobalStateSchema::once(types.get("RGBContract.Details")),
            GS_PRECISION => GlobalStateSchema::once(types.get("RGBContract.Precision")),
            GS_TERMS => GlobalStateSchema::once(types.get("RGBContract.AssetTerms")),
            GS_ISSUED_SUPPLY => GlobalStateSchema::once(types.get("RGBContract.Amount")),
        },
        owned_types: tiny_bmap! {
            OS_ASSET => StateSchema::Fungible(FungibleType::Unsigned64Bit),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: Ty::<SemId>::UNIT.sem_id_unnamed(),
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
        },
        extensions: none!(),
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionSchema {
                metadata: Ty::<SemId>::UNIT.sem_id_unnamed(),
                globals: none!(),
                inputs: tiny_bmap! {
                    OS_ASSET => Occurrences::OnceOrMore
                },
                assignments: tiny_bmap! {
                    OS_ASSET => Occurrences::OnceOrMore
                },
                valencies: none!(),
            }
        },
        script: Script::AluVM(AluScript {
            libs: confined_bmap! { alu_id => alu_lib },
            entry_points: confined_bmap! {
                EntryPoint::ValidateOwnedState(OS_ASSET) => LibSite::with(0, alu_id)
            },
        }),
    }
}

pub fn cfa_rgb25() -> IfaceImpl {
    let schema = cfa_schema();
    let iface = Rgb25::iface(rgb25::Features::none());

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        timestamp: Utc::now().timestamp(),
        developer: none!(), // TODO: Provide issuer information
        script: none!(),
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
    }
}
