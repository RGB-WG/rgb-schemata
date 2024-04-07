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

//! Unique digital asset (UDA) schema implementing RGB21 NFT interface.

use aluvm::isa::opcodes::INSTR_PUTA;
use aluvm::isa::Instr;
use aluvm::library::{Lib, LibSite};
use chrono::Utc;
use rgbstd::interface::{rgb21, IfaceClass, IfaceImpl, NamedField, Rgb21, VerNo};
use rgbstd::schema::{
    GenesisSchema, GlobalStateSchema, Occurrences, Schema, Script, StateSchema, TransitionSchema,
};
use rgbstd::stl::StandardTypes;
use rgbstd::vm::{AluLib, AluScript, EntryPoint, RgbIsa};
use rgbstd::{rgbasm, GlobalStateType, Types};
use strict_types::{SemId, Ty};

use crate::{GS_NOMINAL, GS_TERMS, OS_ASSET, TS_TRANSFER};

const GS_TOKENS: GlobalStateType = GlobalStateType::with(2102);
#[allow(dead_code)]
const GS_ENGRAVINGS: GlobalStateType = GlobalStateType::with(2103);
const GS_ATTACH: GlobalStateType = GlobalStateType::with(2104);

pub fn uda_schema() -> Schema {
    let types = StandardTypes::with(Rgb21::stl());

    let code = rgbasm! {
        // SUBROUTINE 1: genesis validation
        // Initialize registers value
        put     a8[0] ,0x00                 ;
        put     a16[0],0x00                 ;
        put     a16[1],0x00                 ;
        // Read global state into s[1]
        ldg     0x0836,a8[0],s16[1]         ;
        // Extract 128 bits from the beginning of s[1] into r128[1]
        extr    s16[1],r128[1],a16[0]       ;
        // a32[0] now has token index from global state
        spy     a32[0],r128[1]              ;
        // Read owned state into s[1]
        lds     0x0FA0,a16[1],s16[1]        ;
        // Extract 128 bits from the beginning of s[1] into r128[1]
        extr    s16[1],r128[1],a16[0]       ;
        // a32[1] now has token index from allocation
        spy     a32[1],r128[1]              ;
        // check that token indexes match
        eq.n    a32[0],a32[1]               ;
        // if they do, jump to the next check
        jif     0x002D                      ;
        // we need to put a string into first string register which will be used as an error
        // message
        put     s16[0],"the allocated token has an invalid index";
        // fail otherwise
        fail                                ;
        // offset_0x2D (pos 45):
        // Put 4 to a16[0]
        put     a16[0],4                    ;
        // Extract 128 bits starting from the fifth byte of s[1] into r128[0]
        extr    s16[1],r128[1],a16[0]       ;
        // a64[1] now has owned fraction
        spy     a64[1],r128[1]              ;
        // put 1 to a64[0]
        put     a64[0],1                    ;
        // check that owned fraction == 1
        eq.n    a64[0],a64[1]               ;
        // terminate the subroutine
        put     s16[0],"owned fraction is not 1";
        ret                                 ;
        // SUBROUTINE 2: transfer validation
        // offset_0x45 (pos 69):
        // Put 0 to a16[0]
        put     a16[0],0                    ;
        // Read previous state into s[1]
        ldp     0x0FA0,a16[0],s16[1]        ;
        // jump into SUBROUTINE 1 to reuse the code
        jmp     0x0005                      ;
    };
    let alu_lib = AluLib(Lib::assemble::<Instr<RgbIsa>>(&code).unwrap());
    let alu_id = alu_lib.id();
    const FN_GENESIS_OFFSET: u16 = 0x00;
    const FN_TRANSFER_OFFSET: u16 = 0x45;
    assert_eq!(alu_lib.code.as_ref()[0x00], INSTR_PUTA);
    assert_eq!(alu_lib.code.as_ref()[0x2D], INSTR_PUTA);
    assert_eq!(alu_lib.code.as_ref()[FN_TRANSFER_OFFSET as usize], INSTR_PUTA);

    Schema {
        ffv: zero!(),
        flags: none!(),
        types: Types::Strict(types.type_system()),
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalStateSchema::once(types.get("RGBContract.AssetSpec")),
            GS_TERMS => GlobalStateSchema::once(types.get("RGBContract.AssetTerms")),
            GS_TOKENS => GlobalStateSchema::once(types.get("RGB21.TokenData")),
            GS_ATTACH => GlobalStateSchema::once(types.get("RGB21.AttachmentType")),
        },
        owned_types: tiny_bmap! {
            OS_ASSET => StateSchema::Structured(types.get("RGB21.Allocation")),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: Ty::<SemId>::UNIT.sem_id_unnamed(),
            globals: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_TERMS => Occurrences::Once,
                GS_TOKENS => Occurrences::Once,
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
                metadata: Ty::<SemId>::UNIT.sem_id_unnamed(),
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
            libs: confined_bmap! { alu_id => alu_lib },
            entry_points: confined_bmap! {
                EntryPoint::ValidateGenesis => LibSite::with(FN_GENESIS_OFFSET, alu_id),
                EntryPoint::ValidateTransition(TS_TRANSFER) => LibSite::with(FN_TRANSFER_OFFSET, alu_id),
            },
        }),
    }
}

pub fn uda_rgb21() -> IfaceImpl {
    let schema = uda_schema();
    let iface = Rgb21::iface(rgb21::Features::none());

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        timestamp: Utc::now().timestamp(),
        developer: none!(), // TODO: Provide issuer information
        script: none!(),
        global_state: tiny_bset! {
            NamedField::with(GS_NOMINAL, fname!("spec")),
            NamedField::with(GS_TERMS, fname!("terms")),
            NamedField::with(GS_TOKENS, fname!("tokens")),
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
