// RGB schemata by LNP/BP Standards Association
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2023 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2023 LNP/BP Standards Association. All rights reserved.
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

//! Unique digital asset (UDA) schema implementing RGB21 NFT interface.

use rgbstd::interface::{rgb21, rgb21_stl, IfaceImpl, NamedField, NamedType, VerNo};
use rgbstd::schema::{
    GenesisSchema, GlobalStateSchema, Occurrences, Schema, Script, StateSchema, SubSchema,
    TransitionSchema,
};
use rgbstd::stl::StandardTypes;
use rgbstd::vm::AluScript;
use strict_types::{SemId, Ty};

use crate::GS_TIMESTAMP;

const GS_NOMINAL: u16 = 2100;
const GS_CONTRACT: u16 = 2101;
const GS_TOKENS: u16 = 2102;
#[allow(dead_code)]
const GS_ENGRAVINGS: u16 = 2103;
const GS_ATTACH: u16 = 2104;
const OS_ASSET: u16 = 4100;
const TS_TRANSFER: u16 = 10100;

pub fn uda_schema() -> SubSchema {
    let types = StandardTypes::with(rgb21_stl());

    Schema {
        ffv: zero!(),
        subset_of: None,
        type_system: types.type_system(),
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalStateSchema::once(types.get("RGBContract.DivisibleAssetSpec")),
            GS_CONTRACT => GlobalStateSchema::once(types.get("RGBContract.RicardianContract")),
            GS_TIMESTAMP => GlobalStateSchema::once(types.get("RGBContract.Timestamp")),
            GS_TOKENS => GlobalStateSchema::once(types.get("RGB21.TokenData")),
            GS_ATTACH => GlobalStateSchema::once(types.get("RGB21.AttachmentType")),
        },
        owned_types: tiny_bmap! {
            OS_ASSET => StateSchema::Structured(types.get("RGB21.Allocation")),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: Ty::<SemId>::UNIT.id(None),
            globals: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_CONTRACT => Occurrences::Once,
                GS_TIMESTAMP => Occurrences::Once,
                GS_TOKENS => Occurrences::NoneOrOnce,
                GS_ATTACH => Occurrences::NoneOrOnce,
            },
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::Once,
            },
            valencies: none!(),
        },
        extensions: none!(),
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionSchema {
                metadata: Ty::<SemId>::UNIT.id(None),
                globals: none!(),
                inputs: tiny_bmap! {
                    OS_ASSET => Occurrences::Once
                },
                assignments: tiny_bmap! {
                    OS_ASSET => Occurrences::Once
                },
                valencies: none!(),
            }
        },
        script: Script::AluVM(AluScript {
            libs: none!(),
            entry_points: none!(),
        }),
    }
}

pub fn uda_rgb21() -> IfaceImpl {
    let schema = uda_schema();
    let iface = rgb21();

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        global_state: tiny_bset! {
            NamedField::with(GS_NOMINAL, fname!("spec")),
            NamedField::with(GS_CONTRACT, fname!("terms")),
            NamedField::with(GS_TIMESTAMP, fname!("created")),
            NamedField::with(GS_TOKENS, fname!("tokens")),
        },
        assignments: tiny_bset! {
            NamedField::with(OS_ASSET, fname!("beneficiary")),
        },
        valencies: none!(),
        transitions: tiny_bset! {
            NamedType::with(TS_TRANSFER, tn!("Transfer")),
        },
        extensions: none!(),
    }
}
