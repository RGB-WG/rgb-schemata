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

//! Unique digital asset (UDA) schema.

use aluvm::isa::opcodes::{INSTR_EXTR, INSTR_PUTA};
use aluvm::isa::Instr;
use aluvm::library::{Lib, LibSite};
use amplify::confinement::Confined;
use rgbstd::contract::{
    AssignmentsFilter, ContractData, DataAllocation, IssuerWrapper, SchemaWrapper,
};
use rgbstd::persistence::{ContractStateRead, MemContract};
use rgbstd::schema::{
    AssignmentDetails, GenesisSchema, GlobalStateSchema, Occurrences, Schema, TransitionSchema,
};
use rgbstd::stl::{rgb_contract_stl, AssetSpec, ContractTerms, StandardTypes, TokenData};
use rgbstd::validation::Scripts;
use rgbstd::vm::opcodes::INSTR_LDG;
use rgbstd::vm::RgbIsa;
use rgbstd::{rgbasm, GlobalDetails, OwnedStateSchema, SchemaId, TransitionDetails};
use strict_types::TypeSystem;

use crate::{
    ERRNO_NON_EQUAL_IN_OUT, ERRNO_NON_FRACTIONAL, GS_ATTACH, GS_NOMINAL, GS_TERMS, GS_TOKENS,
    OS_ASSET, TS_TRANSFER,
};

pub const UDA_SCHEMA_ID: SchemaId = SchemaId::from_array([
    0xff, 0xaa, 0xe3, 0xca, 0x67, 0xf7, 0x19, 0x31, 0x3c, 0xe3, 0x49, 0x5b, 0xe4, 0x9a, 0x17, 0x9b,
    0x66, 0x85, 0xc0, 0x4f, 0x1e, 0x58, 0x29, 0x37, 0x98, 0x28, 0xce, 0x7f, 0xe9, 0x94, 0xce, 0xd1,
]);

pub const FN_GENESIS_OFFSET: u16 = 4 + 4 + 3;
pub const FN_TRANSFER_OFFSET: u16 = 0;
pub const FN_SHARED_OFFSET: u16 = FN_GENESIS_OFFSET + 4 + 4 + 4;

fn uda_standard_types() -> StandardTypes { StandardTypes::with(rgb_contract_stl()) }

fn uda_lib() -> Lib {
    let code = rgbasm! {
        // SUBROUTINE 2: Transfer validation
        // Put 0 to a16[0]
        put     a16[0],0;
        // Read previous state into s16[0]
        ldp     OS_ASSET,a16[0],s16[0];
        // jump into SUBROUTINE 3 to reuse the code
        jmp     FN_SHARED_OFFSET;

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
    Lib::assemble::<Instr<RgbIsa<MemContract>>>(&code).expect("wrong unique digital asset script")
}

fn uda_schema() -> Schema {
    let types = uda_standard_types();

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
        name: tn!("UniqueDigitalAsset"),
        meta_types: none!(),
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.AssetSpec")),
                name: fname!("spec"),
            },
            GS_TERMS => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.ContractTerms")),
                name: fname!("terms"),
            },
            GS_TOKENS => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.TokenData")),
                name: fname!("tokens"),
            },
            GS_ATTACH => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.AttachmentType")),
                name: fname!("attachmentTypes"),
            },
        },
        owned_types: tiny_bmap! {
            OS_ASSET => AssignmentDetails {
                owned_state_schema: OwnedStateSchema::Structured(types.get("RGBContract.Allocation")),
                name: fname!("assetOwner"),
                default_transition: TS_TRANSFER,
            }
        },
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
            validator: Some(LibSite::with(FN_GENESIS_OFFSET, alu_id)),
        },
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionDetails {
                transition_schema: TransitionSchema {
                    metadata: none!(),
                    globals: none!(),
                    inputs: tiny_bmap! {
                        OS_ASSET => Occurrences::Once
                    },
                    assignments: tiny_bmap! {
                        OS_ASSET => Occurrences::Once
                    },
                    validator: Some(LibSite::with(FN_TRANSFER_OFFSET, alu_id)),
                },
                name: fname!("transfer"),
            }
        },
        default_assignment: Some(OS_ASSET),
    }
}

#[derive(Default)]
pub struct UniqueDigitalAsset;

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub struct UdaWrapper<S: ContractStateRead>(ContractData<S>);

impl IssuerWrapper for UniqueDigitalAsset {
    type Wrapper<S: ContractStateRead> = UdaWrapper<S>;

    fn schema() -> Schema { uda_schema() }

    fn types() -> TypeSystem { uda_standard_types().type_system(uda_schema()) }

    fn scripts() -> Scripts {
        let lib = uda_lib();
        Confined::from_checked(bmap! { lib.id() => lib })
    }
}

impl<S: ContractStateRead> SchemaWrapper<S> for UdaWrapper<S> {
    fn with(data: ContractData<S>) -> Self {
        if data.schema.schema_id() != UDA_SCHEMA_ID {
            panic!("the provided schema is not UDA");
        }
        Self(data)
    }
}

impl<S: ContractStateRead> UdaWrapper<S> {
    pub fn spec(&self) -> AssetSpec {
        let strict_val = &self
            .0
            .global("spec")
            .next()
            .expect("UDA requires global state `spec` to have at least one item");
        AssetSpec::from_strict_val_unchecked(strict_val)
    }

    pub fn contract_terms(&self) -> ContractTerms {
        let strict_val = &self
            .0
            .global("terms")
            .next()
            .expect("UDA requires global state `terms` to have at least one item");
        ContractTerms::from_strict_val_unchecked(strict_val)
    }

    pub fn token_data(&self) -> TokenData {
        let strict_val = &self
            .0
            .global("tokens")
            .next()
            .expect("UDA requires global state `tokens` to have at least one item");
        TokenData::from_strict_val_unchecked(strict_val)
    }

    pub fn allocations<'c>(
        &'c self,
        filter: impl AssignmentsFilter + 'c,
    ) -> impl Iterator<Item = DataAllocation> + 'c {
        self.0.data_raw(OS_ASSET, filter).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn schema_id() {
        let schema_id = uda_schema().schema_id();
        eprintln!("{:#04x?}", schema_id.to_byte_array());
        assert_eq!(UDA_SCHEMA_ID, schema_id);
    }
}
