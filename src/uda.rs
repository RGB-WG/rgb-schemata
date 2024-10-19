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

use aluvm::library::{Lib, LibSite};
use amplify::confinement::Confined;
use ifaces::stl::StandardTypes;
use ifaces::{IssuerWrapper, LNPBP_IDENTITY, Rgb21};
use rgbstd::interface::{IfaceClass, IfaceImpl, NamedField, NamedVariant, StateAbi, VerNo};
use rgbstd::schema::{GenesisSchema, GlobalStateSchema, Occurrences, Schema, TransitionSchema};
use rgbstd::validation::Scripts;
use rgbstd::{Identity, OwnedStateSchema, rgbasm};
use strict_types::TypeSystem;

use crate::{
    ERRNO_NON_EQUAL_IN_OUT, ERRNO_NON_FRACTIONAL, GS_ATTACH, GS_NOMINAL, GS_TERMS, GS_TOKENS,
    OS_ASSET, TS_TRANSFER,
};

pub const FN_GENESIS_OFFSET: u16 = 0x00;
pub const FN_TRANSFER_OFFSET: u16 = 0x0E;
pub const FN_SHARED_OFFSET: u16 = 0x19;

fn uda_lib() -> Lib {
    const TOKEN: u16 = OS_ASSET.to_u16();
    const ISSUE: u16 = GS_TOKENS.to_u16();

    rgbasm! {
        // SUBROUTINE 1: Genesis validation
        put     a16[0],0;                       // zero constant
        put     a16[15],ISSUE;                  // global state type
        ld.g    s16[0],a16[15],a16[0];          // load token declaration
        jmp     FN_SHARED_OFFSET;               // jump into SUBROUTINE 3 to reuse the code

        // SUBROUTINE 2: Transfer validation
        put     a16[0],0;                       // zero constant
        put     a16[16],TOKEN;                  // owned state type
        ld.i    s16[0],a16[16],a16[0];          // load spent token

        // SUBROUTINE 3: Shared code
        extr    s16[0],a32[0],a16[0];           // 32 bits from the beginning of s16[0] to a32[0]
        put     a16[16],TOKEN;                  // owned state type
        ld.o    s16[1],a16[16],a16[0];          // read allocation into s16[1]
        extr    s16[1],a32[1],a16[0];           // 32 bits from the beginning of s16[0] to a32[1]
        put     a8[0],ERRNO_NON_EQUAL_IN_OUT;   // set failure code
        eq.n    a32[0],a32[1];                  // check that token indexes match
        test;                                   // fail if they don't

        put     a8[0],ERRNO_NON_FRACTIONAL;     // set failure code
        put     a16[2],4;                       // offset for the token fractions into a16[2]
        extr    s16[1],a64[0],a16[2];           // 64 bit from the fifth byte of s16[1] to a64[0]
        put     a64[1],1;
        eq.n    a64[0],a64[1];                  // check that owned fraction == 1
        test;                                   // fail if not
    }
}

fn uda_schema() -> Schema {
    let types = StandardTypes::with(Rgb21::NONE.stl());

    let alu_lib = uda_lib();
    let alu_id = alu_lib.id();

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
            OS_ASSET => OwnedStateSchema::from(types.get("RGBContract.NftAllocation")),
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
    let lib_id = uda_lib().id();

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: Rgb21::NONE.iface_id(),
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
        state_abi: StateAbi {
            reg_input: LibSite::with(0, lib_id),
            reg_output: LibSite::with(0, lib_id),
            calc_output: LibSite::with(0, lib_id),
            calc_change: LibSite::with(0, lib_id),
        },
    }
}

#[derive(Default)]
pub struct UniqueDigitalAsset;

impl IssuerWrapper for UniqueDigitalAsset {
    type IssuingIface = Rgb21;
    const FEATURES: Rgb21 = Rgb21::NONE;

    fn schema() -> Schema { uda_schema() }
    fn issue_impl() -> IfaceImpl { uda_rgb21() }

    fn types() -> TypeSystem { StandardTypes::with(Self::FEATURES.stl()).type_system() }

    fn scripts() -> Scripts {
        let lib = uda_lib();
        Confined::from_checked(bmap! { lib.id() => lib })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iimpl_check() {
        let iface = UniqueDigitalAsset::FEATURES.iface();
        if let Err(err) = uda_rgb21().check(&iface, &uda_schema()) {
            for e in err {
                eprintln!("{e}");
            }
            panic!("invalid UDA RGB21 interface implementation");
        }
    }
}
