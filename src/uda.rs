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

use aluvm::isa::opcodes::INSTR_PUTA;
use aluvm::isa::Instr;
use aluvm::library::{Lib, LibSite};
use rgbstd::interface::{rgb21, rgb21_stl, IfaceImpl, NamedField, NamedType, VerNo};
use rgbstd::schema::{
    GenesisSchema, GlobalStateSchema, Occurrences, Schema, Script, StateSchema, SubSchema,
    TransitionSchema,
};
use rgbstd::stl::StandardTypes;
use rgbstd::vm::opcodes::INSTR_LDP;
use rgbstd::vm::{AluScript, EntryPoint, RgbIsa};
use rgbstd::{rgbasm, GlobalStateType};
use strict_types::{SemId, Ty};

use crate::{GS_NOMINAL, GS_TIMESTAMP, OS_ASSET, TS_TRANSFER};

const GS_CONTRACT: GlobalStateType = GlobalStateType::with(2101);
const GS_TOKENS: GlobalStateType = GlobalStateType::with(2102);
#[allow(dead_code)]
const GS_ENGRAVINGS: GlobalStateType = GlobalStateType::with(2103);
const GS_ATTACH: GlobalStateType = GlobalStateType::with(2104);

pub fn uda_schema() -> SubSchema {
    let types = StandardTypes::with(rgb21_stl());

    let code = rgbasm! {
        // SUBROUTINE 1: genesis validation
        // TODO: Use index for global state from register
        // Read global state into s[1]
        ldg     0x0836,0,s16[1]             ;
        // Put 0 to a16[0]
        put     a16[0],0x00                 ;
        // Extract 128 bits from the beginning of s[1] into r128[1]
        extr    s16[1],r128[1],a16[0]       ;
        // a32[0] now has token index from global state
        spy     a32[0],r128[1]              ;
        // Read owned state into s[1]
        lds     0x0FA0,0,s16[1]             ;
        // Extract 128 bits from the beginning of s[1] into r128[1]
        extr    s16[1],r128[1],a16[0]       ;
        // a32[1] now has token index from allocation
        spy     a32[1],r128[1]              ;
        // check that token indexes match
        eq.n    a32[0],a32[1]               ;
        // if they do, jump to the next check
        jif     0x0028                      ;
        // we need to put a string into first string register which will be used as an error
        // message
        put     s16[0],"the allocated token has an invalid index";
        // fail otherwise
        fail                                ;
        // offset_0x28:
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
        // offset_0x40:
        // Read previous state into s[1]
        ldp     0x0FA0,0,s16[1]             ;
        // jump into SUBROUTINE 1 to reuse the code
        jmp     0x0005                      ;
    };
    let alu_lib = Lib::assemble::<Instr<RgbIsa>>(&code).unwrap();
    let alu_id = alu_lib.id();
    const FN_GENESIS_OFFSET: u16 = 0;
    const FN_TRANSFER_OFFSET: u16 = 0x40;
    assert_eq!(alu_lib.code.as_ref()[0x05], INSTR_PUTA);
    assert_eq!(alu_lib.code.as_ref()[0x28], INSTR_PUTA);
    assert_eq!(alu_lib.code.as_ref()[FN_TRANSFER_OFFSET as usize], INSTR_LDP);

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
    let iface = rgb21();

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        script: none!(),
        global_state: tiny_bset! {
            NamedField::with(GS_NOMINAL, fname!("spec")),
            NamedField::with(GS_CONTRACT, fname!("terms")),
            NamedField::with(GS_TIMESTAMP, fname!("created")),
            NamedField::with(GS_TOKENS, fname!("tokens")),
        },
        assignments: tiny_bset! {
            NamedField::with(OS_ASSET, fname!("assetOwner")),
        },
        valencies: none!(),
        transitions: tiny_bset! {
            NamedType::with(TS_TRANSFER, tn!("Transfer")),
        },
        extensions: none!(),
    }
}
