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

use aluvm::isa::opcodes::{INSTR_EXTR, INSTR_PUTA};
use aluvm::isa::Instr;
use aluvm::library::{Lib, LibSite};
use ifaces::{rgb21, IssuerWrapper, Rgb21, LNPBP_IDENTITY};
use rgbstd::interface::{IfaceClass, IfaceImpl, NamedField, NamedVariant, VerNo};
use rgbstd::schema::{GenesisSchema, GlobalStateSchema, Occurrences, Schema, TransitionSchema};
use rgbstd::stl::StandardTypes;
use rgbstd::validation::Scripts;
use rgbstd::vm::opcodes::INSTR_LDG;
use rgbstd::vm::RgbIsa;
use rgbstd::{rgbasm, Identity, OwnedStateSchema};
use strict_types::TypeSystem;

use crate::{
    ERRNO_NON_EQUAL_IN_OUT, ERRNO_NON_FRACTIONAL, GS_ATTACH, GS_NOMINAL, GS_TERMS, GS_TOKENS,
    OS_ASSET, TS_TRANSFER,
};

pub const FN_GENESIS_OFFSET: u16 = 4 + 4 + 3;
pub const FN_TRANSFER_OFFSET: u16 = 0;
pub const FN_SHARED_OFFSET: u16 = FN_GENESIS_OFFSET + 4 + 4 + 4;

fn uda_lib() -> Lib {
    let code = rgbasm! {
        // SUBROUTINE 2: Transfer validation
        // Put 0 to a16[0]
        put     a16[0],0;
        // Read previous state into s16[0]
        ldp     OS_ASSET,a16[0],s16[0];
        // jump into SUBROUTINE 1 to reuse the code
        jmp     0x0005;

        // SUBROUTINE 1: Genesis validation
        // Set offset to read state from strings
        put     a16[0],0x00;
        // Set which state index to read
        put     a8[1],0x00;
        // Read global state into s16[0]
        ldg     GS_TOKENS,a8[1],s16[0];

        // SUBROUTINE 3: Shared code
        // Set errno
        put     a8[0],ERRNO_NON_EQUAL_IN_OUT;
        // Extract 128 bits from the beginning of s16[0] into a32[0]
        extr    s16[0],a32[0],a16[0];
        // Set which state index to read
        put     a16[1],0x00;
        // Read owned state into s16[1]
        lds     OS_ASSET,a16[1],s16[1];
        // Extract 128 bits from the beginning of s16[1] into a32[1]
        extr    s16[1],a32[1],a16[0];
        // Check that token indexes match
        eq.n    a32[0],a32[1];
        // Fail if they don't
        test;

        // Set errno
        put     a8[0],ERRNO_NON_FRACTIONAL;
        // Put offset for the data into a16[2]
        put     a16[2],4;
        // Extract 128 bits starting from the fifth byte of s16[1] into a64[0]
        extr    s16[1],a64[0],a16[2];
        // Check that owned fraction == 1
        put     a64[1],1;
        eq.n    a64[0],a64[1];
        // Fail if not
        test;
    };
    Lib::assemble::<Instr<RgbIsa>>(&code).expect("wrong unique digital asset script")
}

fn uda_schema() -> Schema {
    let types = StandardTypes::with(Rgb21::stl());

    let alu_lib = uda_lib();
    let alu_id = alu_lib.id();
    let code = alu_lib.code.as_ref();
    assert_eq!(code[FN_GENESIS_OFFSET as usize], INSTR_PUTA);
    assert_eq!(code[FN_GENESIS_OFFSET as usize + 8], INSTR_LDG);
    assert_eq!(code[FN_TRANSFER_OFFSET as usize], INSTR_PUTA);
    assert_eq!(code[FN_SHARED_OFFSET as usize], INSTR_PUTA);
    assert_eq!(code[FN_SHARED_OFFSET as usize + 4], INSTR_EXTR);

    Schema {
        ffv: zero!(),
        flags: none!(),
        name: tn!("UniqueDigitalAsset"),
        timestamp: 1713343888,
        developer: Identity::from(LNPBP_IDENTITY),
        meta_types: none!(),
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalStateSchema::once(types.get("RGBContract.AssetSpec")),
            GS_TERMS => GlobalStateSchema::once(types.get("RGBContract.ContractTerms")),
            GS_TOKENS => GlobalStateSchema::once(types.get("RGB21.TokenData")),
            GS_ATTACH => GlobalStateSchema::once(types.get("RGB21.AttachmentType")),
        },
        owned_types: tiny_bmap! {
            OS_ASSET => OwnedStateSchema::Structured(types.get("RGBContract.Allocation")),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: none!(),
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
            validator: Some(LibSite::with(FN_GENESIS_OFFSET, alu_id)),
        },
        extensions: none!(),
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionSchema {
                metadata: none!(),
                globals: none!(),
                inputs: tiny_bmap! {
                    OS_ASSET => Occurrences::Once
                },
                assignments: tiny_bmap! {
                    OS_ASSET => Occurrences::Once
                },
                valencies: none!(),
                validator: Some(LibSite::with(FN_TRANSFER_OFFSET, alu_id)),
            }
        },
        reserved: none!(),
    }
}

fn uda_rgb21() -> IfaceImpl {
    let schema = uda_schema();
    let iface = Rgb21::iface(rgb21::Features::NONE);

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
            NamedField::with(GS_TOKENS, fname!("tokens")),
            NamedField::with(GS_ATTACH, fname!("attachmentTypes")),
        },
        assignments: tiny_bset! {
            NamedField::with(OS_ASSET, fname!("assetOwner")),
        },
        valencies: none!(),
        transitions: tiny_bset! {
            NamedField::with(TS_TRANSFER, fname!("transfer")),
        },
        extensions: none!(),
        errors: tiny_bset! {
            NamedVariant::with(ERRNO_NON_FRACTIONAL, vname!("nonFractionalToken")),
            NamedVariant::with(ERRNO_NON_EQUAL_IN_OUT, vname!("unknownToken")),
        },
    }
}

pub struct UniqueDigitalAsset;

impl IssuerWrapper for UniqueDigitalAsset {
    const FEATURES: rgb21::Features = rgb21::Features::NONE;
    type IssuingIface = Rgb21;

    fn schema() -> Schema { uda_schema() }
    fn issue_impl() -> IfaceImpl { uda_rgb21() }

    fn types() -> TypeSystem { StandardTypes::with(Rgb21::stl()).type_system() }

    fn scripts() -> Scripts {
        let lib = uda_lib();
        confined_bmap! { lib.id() => lib }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iimpl_check() {
        let iface = Rgb21::iface(UniqueDigitalAsset::FEATURES);
        if let Err(err) = uda_rgb21().check(&iface, &uda_schema()) {
            for e in err {
                eprintln!("{e}");
            }
            panic!("invalid UDA RGB21 interface implementation");
        }
    }
}
