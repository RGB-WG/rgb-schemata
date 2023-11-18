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

//! Non-Inflatable Assets (NIA) schema implementing RGB20 fungible assets
//! interface.

use aluvm::data::ByteStr;
use aluvm::isa::{BytesOp, ControlFlowOp, Instr};
use aluvm::library::{Lib, LibSite};
use aluvm::reg::RegS;
use rgbstd::interface::{rgb20, rgb20_stl, IfaceImpl, NamedField, NamedType, VerNo};
use rgbstd::schema::{
    FungibleType, GenesisSchema, GlobalStateSchema, Occurrences, Schema, Script, StateSchema,
    SubSchema, TransitionSchema,
};
use rgbstd::stl::StandardTypes;
use rgbstd::vm::opcodes::{INSTR_PCCS, INSTR_PCVS};
use rgbstd::vm::{AluScript, ContractOp, EntryPoint, RgbIsa};
use strict_types::{SemId, Ty};

use crate::{GS_DATA, GS_ISSUED_SUPPLY, GS_NOMINAL, GS_TIMESTAMP, OS_ASSET, TS_TRANSFER};

pub fn nia_schema() -> SubSchema {
    let types = StandardTypes::with(rgb20_stl());

    let code: [Instr<RgbIsa>; 5] = [
        // SUBROUTINE 1: genesis validation
        // Before doing a check we need to put a string into first string register which will be
        // used as an error message in case of the check failure.
        Instr::Bytes(BytesOp::Put(
            RegS::from(0),
            Box::new(ByteStr::with(
                "the issued amounts do not match between the reported global state and \
                 confidential owned state",
            )),
            true,
        )),
        // Checking pedersen commitments against reported amount of issued assets present in the
        // global state.
        Instr::ExtensionCodes(RgbIsa::Contract(ContractOp::PcCs(OS_ASSET, GS_ISSUED_SUPPLY))),
        // If the check succeeds we need to terminate the subroutine.
        Instr::ControlFlow(ControlFlowOp::Ret),
        // SUBROUTINE 2: transfer validation
        Instr::Bytes(BytesOp::Put(
            RegS::from(0),
            Box::new(ByteStr::with(
                "the sum of input amounts do not match the sum of the output amounts",
            )),
            true,
        )),
        // Checking that the sum of pedersen commitments in inputs is equal to the sum in outputs.
        Instr::ExtensionCodes(RgbIsa::Contract(ContractOp::PcVs(OS_ASSET))),
    ];
    let alu_lib = Lib::assemble::<Instr<RgbIsa>>(&code).unwrap();
    let alu_id = alu_lib.id();
    const FN_GENESIS_OFFSET: u16 = 0;
    const FN_TRANSFER_OFFSET: u16 = 6 + 5 + 1;
    assert_eq!(alu_lib.code.as_ref()[FN_GENESIS_OFFSET as usize + 6], INSTR_PCCS);
    assert_eq!(alu_lib.code.as_ref()[FN_TRANSFER_OFFSET as usize + 6], INSTR_PCVS);

    Schema {
        ffv: zero!(),
        subset_of: None,
        type_system: types.type_system(),
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalStateSchema::once(types.get("RGBContract.DivisibleAssetSpec")),
            GS_DATA => GlobalStateSchema::once(types.get("RGBContract.ContractData")),
            GS_TIMESTAMP => GlobalStateSchema::once(types.get("RGBContract.Timestamp")),
            GS_ISSUED_SUPPLY => GlobalStateSchema::once(types.get("RGBContract.Amount")),
        },
        owned_types: tiny_bmap! {
            OS_ASSET => StateSchema::Fungible(FungibleType::Unsigned64Bit),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: Ty::<SemId>::UNIT.id(None),
            globals: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_DATA => Occurrences::Once,
                GS_TIMESTAMP => Occurrences::Once,
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
                metadata: Ty::<SemId>::UNIT.id(None),
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
                EntryPoint::ValidateGenesis => LibSite::with(FN_GENESIS_OFFSET, alu_id),
                EntryPoint::ValidateTransition(TS_TRANSFER) => LibSite::with(FN_TRANSFER_OFFSET, alu_id),
            },
        }),
    }
}

pub fn nia_rgb20() -> IfaceImpl {
    let schema = nia_schema();
    let iface = rgb20();

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        script: none!(),
        global_state: tiny_bset! {
            NamedField::with(GS_NOMINAL, fname!("spec")),
            NamedField::with(GS_DATA, fname!("data")),
            NamedField::with(GS_TIMESTAMP, fname!("created")),
            NamedField::with(GS_ISSUED_SUPPLY, fname!("issuedSupply")),
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
