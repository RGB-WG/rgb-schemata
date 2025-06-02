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

//! Non-Inflatable Assets (NIA) schema.

use aluvm::isa::opcodes::INSTR_PUTA;
use aluvm::isa::Instr;
use aluvm::library::{Lib, LibSite};
use amplify::confinement::Confined;
use rgbstd::contract::{
    AssignmentsFilter, ContractData, FungibleAllocation, IssuerWrapper, SchemaWrapper,
};
use rgbstd::persistence::{ContractStateRead, MemContract};
use rgbstd::schema::{
    AssignmentDetails, FungibleType, GenesisSchema, GlobalDetails, GlobalStateSchema, Occurrences,
    OwnedStateSchema, Schema, TransitionSchema,
};
use rgbstd::stl::{rgb_contract_stl, AssetSpec, ContractTerms, StandardTypes};
use rgbstd::validation::Scripts;
use rgbstd::vm::opcodes::INSTR_SVS;
use rgbstd::vm::RgbIsa;
use rgbstd::{rgbasm, Amount, SchemaId, TransitionDetails};
use strict_types::TypeSystem;

use crate::{
    ERRNO_ISSUED_MISMATCH, ERRNO_NON_EQUAL_IN_OUT, GS_ISSUED_SUPPLY, GS_NOMINAL, GS_TERMS,
    OS_ASSET, TS_TRANSFER,
};

pub const NIA_SCHEMA_ID: SchemaId = SchemaId::from_array([
    0x45, 0x68, 0x70, 0x51, 0xf4, 0xcc, 0xa6, 0xe3, 0xf6, 0x65, 0xfc, 0x75, 0xfe, 0x3e, 0x27, 0xb3,
    0x00, 0x80, 0x34, 0x67, 0x89, 0xad, 0x83, 0xaa, 0x0d, 0xc2, 0x9e, 0x95, 0xa3, 0x15, 0xe3, 0x35,
]);

pub(crate) fn nia_lib() -> Lib {
    let code = rgbasm! {
        // SUBROUTINE Transfer validation
        // Set errno
        put     a8[0],ERRNO_NON_EQUAL_IN_OUT;
        // Checking that the sum of inputs is equal to the sum of outputs.
        svs     OS_ASSET;
        test;
        ret;

        // SUBROUTINE Genesis validation
        // Checking genesis assignments amount against reported amount of issued assets present in
        // the global state.
        put     a8[0],ERRNO_ISSUED_MISMATCH;
        put     a8[1],0;
        put     a16[0],0;
        // Read global state into s16[0]
        ldg     GS_ISSUED_SUPPLY,a8[1],s16[0];
        // Extract 64 bits from the beginning of s16[0] into a64[0]
        // NB: if the global state is invalid, we will fail here and fail the validation
        extr    s16[0],a64[0],a16[0];
        // verify sum of outputs against a64[0] value
        sas     OS_ASSET;
        test;
        ret;
    };
    Lib::assemble::<Instr<RgbIsa<MemContract>>>(&code).expect("wrong non-inflatable asset script")
}
pub(crate) const FN_NIA_GENESIS_OFFSET: u16 = 4 + 3 + 2;
pub(crate) const FN_NIA_TRANSFER_OFFSET: u16 = 0;

fn nia_standard_types() -> StandardTypes { StandardTypes::with(rgb_contract_stl()) }

fn nia_schema() -> Schema {
    let types = nia_standard_types();

    let alu_lib = nia_lib();
    let alu_id = alu_lib.id();
    assert_eq!(alu_lib.code.as_ref()[FN_NIA_TRANSFER_OFFSET as usize + 4], INSTR_SVS);
    assert_eq!(alu_lib.code.as_ref()[FN_NIA_GENESIS_OFFSET as usize], INSTR_PUTA);
    assert_eq!(alu_lib.code.as_ref()[FN_NIA_GENESIS_OFFSET as usize + 4], INSTR_PUTA);
    assert_eq!(alu_lib.code.as_ref()[FN_NIA_GENESIS_OFFSET as usize + 8], INSTR_PUTA);

    Schema {
        ffv: zero!(),
        name: tn!("NonInflatableAsset"),
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
            GS_ISSUED_SUPPLY => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.Amount")),
                name: fname!("issuedSupply"),
            },
        },
        owned_types: tiny_bmap! {
            OS_ASSET => AssignmentDetails {
                owned_state_schema: OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
                name: fname!("assetOwner"),
                default_transition: TS_TRANSFER,
            }
        },
        genesis: GenesisSchema {
            metadata: none!(),
            globals: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_TERMS => Occurrences::Once,
                GS_ISSUED_SUPPLY => Occurrences::Once,
            },
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::OnceOrMore,
            },
            validator: Some(LibSite::with(FN_NIA_GENESIS_OFFSET, alu_id)),
        },
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionDetails {
                transition_schema: TransitionSchema {
                    metadata: none!(),
                    globals: none!(),
                    inputs: tiny_bmap! {
                        OS_ASSET => Occurrences::OnceOrMore
                    },
                    assignments: tiny_bmap! {
                        OS_ASSET => Occurrences::OnceOrMore
                    },
                    validator: Some(LibSite::with(FN_NIA_TRANSFER_OFFSET, alu_id))
                },
                name: fname!("transfer"),
            }
        },
        default_assignment: Some(OS_ASSET),
    }
}

#[derive(Default)]
pub struct NonInflatableAsset;

impl IssuerWrapper for NonInflatableAsset {
    type Wrapper<S: ContractStateRead> = NiaWrapper<S>;

    fn schema() -> Schema { nia_schema() }

    fn types() -> TypeSystem { nia_standard_types().type_system(nia_schema()) }

    fn scripts() -> Scripts {
        let lib = nia_lib();
        Confined::from_checked(bmap! { lib.id() => lib })
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub struct NiaWrapper<S: ContractStateRead>(ContractData<S>);

impl<S: ContractStateRead> SchemaWrapper<S> for NiaWrapper<S> {
    fn with(data: ContractData<S>) -> Self {
        if data.schema.schema_id() != NIA_SCHEMA_ID {
            panic!("the provided schema is not NIA");
        }
        Self(data)
    }
}

impl<S: ContractStateRead> NiaWrapper<S> {
    pub fn spec(&self) -> AssetSpec {
        let strict_val = &self
            .0
            .global("spec")
            .next()
            .expect("NIA requires global state `spec` to have at least one item");
        AssetSpec::from_strict_val_unchecked(strict_val)
    }

    pub fn contract_terms(&self) -> ContractTerms {
        let strict_val = &self
            .0
            .global("terms")
            .next()
            .expect("NIA requires global state `terms` to have at least one item");
        ContractTerms::from_strict_val_unchecked(strict_val)
    }

    pub fn total_issued_supply(&self) -> Amount {
        self.0
            .global("issuedSupply")
            .map(|amount| Amount::from_strict_val_unchecked(&amount))
            .sum()
    }

    pub fn allocations<'c>(
        &'c self,
        filter: impl AssignmentsFilter + 'c,
    ) -> impl Iterator<Item = FungibleAllocation> + 'c {
        self.0.fungible_raw(OS_ASSET, filter).unwrap()
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bp::seals::txout::BlindSeal;
    use bp::Txid;
    use rgbstd::containers::{BuilderSeal, ConsignmentExt};
    use rgbstd::contract::*;
    use rgbstd::invoice::Precision;
    use rgbstd::stl::*;
    use rgbstd::*;

    use super::*;

    #[test]
    fn schema_id() {
        let schema_id = nia_schema().schema_id();
        eprintln!("{:#04x?}", schema_id.to_byte_array());
        assert_eq!(NIA_SCHEMA_ID, schema_id);
    }

    #[test]
    fn deterministic_contract_id() {
        let created_at = 1713261744;
        let terms = ContractTerms {
            text: RicardianContract::default(),
            media: None,
        };
        let spec = AssetSpec {
            ticker: Ticker::from("TICKER"),
            name: Name::from("NAME"),
            details: None,
            precision: Precision::try_from(2).unwrap(),
        };
        let issued_supply = 999u64;
        let seal: BlindSeal<Txid> = GenesisSeal::from(BlindSeal::with_blinding(
            Txid::from_str("8d54c98d4c29a1ec4fd90635f543f0f7a871a78eb6a6e706342f831d92e3ba19")
                .unwrap(),
            0,
            654321,
        ));

        let builder = ContractBuilder::with(
            Identity::default(),
            NonInflatableAsset::schema(),
            NonInflatableAsset::types(),
            NonInflatableAsset::scripts(),
            ChainNet::BitcoinTestnet4,
        )
        .add_global_state("spec", spec)
        .unwrap()
        .add_global_state("terms", terms)
        .unwrap()
        .add_global_state("issuedSupply", Amount::from(issued_supply))
        .unwrap()
        .add_fungible_state("assetOwner", BuilderSeal::from(seal), issued_supply)
        .unwrap();

        let contract = builder.issue_contract_raw(created_at).unwrap();

        assert_eq!(
            contract.contract_id().to_string(),
            s!("rgb:663wqep~-0pVYnjS-ieA0N3r-58wUTIY-zgCGO_1-QQkuMMs")
        );
    }
}
