// RGB schemata by LNP/BP Standards Association
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2025 by
//     Stefano Pellegrini <stefano.pellegrini@bitfinex.com>
//
// Copyright (C) 2025 LNP/BP Standards Association. All rights reserved.
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

//! Inflatable Fungible Assets (IFA) schema.

use aluvm::isa::Instr;
use aluvm::library::{Lib, LibSite};
use amplify::confinement::Confined;
use rgbstd::contract::{
    AssignmentsFilter, ContractData, FungibleAllocation, IssuerWrapper, RightsAllocation,
    SchemaWrapper,
};
use rgbstd::persistence::{ContractStateRead, MemContract};
use rgbstd::schema::{
    AssignmentDetails, FungibleType, GenesisSchema, GlobalStateSchema, Occurrences,
    OwnedStateSchema, Schema, TransitionSchema,
};
use rgbstd::stl::{rgb_contract_stl, AssetSpec, ContractTerms, OpidRejectUrl, StandardTypes};
use rgbstd::validation::Scripts;
use rgbstd::vm::RgbIsa;
use rgbstd::{rgbasm, Amount, GlobalDetails, MetaDetails, SchemaId, TransitionDetails};
use strict_types::TypeSystem;

use crate::{
    ERRNO_INFLATION_EXCEEDS_ALLOWANCE, ERRNO_INFLATION_MISMATCH, ERRNO_ISSUED_MISMATCH,
    ERRNO_NON_EQUAL_IN_OUT, GS_ISSUED_SUPPLY, GS_MAX_SUPPLY, GS_NOMINAL, GS_OPID_REJECT_URL,
    GS_TERMS, MS_ALLOWED_INFLATION, OS_ASSET, OS_INFLATION, OS_REPLACE, TS_BURN, TS_INFLATION,
    TS_REPLACE, TS_TRANSFER,
};

pub const IFA_SCHEMA_ID: SchemaId = SchemaId::from_array([
    0xff, 0x79, 0x94, 0x70, 0x37, 0xc1, 0x38, 0xd7, 0x5f, 0xa3, 0x27, 0x47, 0x20, 0xde, 0x9a, 0xa2,
    0x77, 0xae, 0xbe, 0x9d, 0xcc, 0xce, 0xda, 0x83, 0xdf, 0x47, 0xc7, 0xe3, 0xcc, 0x37, 0xc6, 0x5d,
]);

pub(crate) fn ifa_lib_genesis() -> Lib {
    #[allow(clippy::diverging_sub_expression)]
    let code = rgbasm! {
        // Set common offsets
        put     a8[1],0;
        put     a16[0],0;

        // Check reported issued supply against sum of asset allocations in output
        put     a8[0],ERRNO_ISSUED_MISMATCH;  // set errno
        ldg     GS_ISSUED_SUPPLY,a8[1],s16[0];  // read issued supply global state
        extr    s16[0],a64[0],a16[0];  // and store it in a64[0]
        sas     OS_ASSET;  // check sum of assets assignments in output equals a64[0]
        test;

        // Check that sum of inflation rights = max supply - issued supply
        put     a8[0],ERRNO_INFLATION_MISMATCH;  // set errno
        ldg     GS_MAX_SUPPLY,a8[1],s16[1];  // read max supply global state
        extr    s16[1],a64[1],a16[0];  // and store it in a64[1]
        sub.uc  a64[1],a64[0];  // issued supply is still in a64[0], result overwrites a64[0]
        test;  // fails if result is <0
        sas     OS_INFLATION;  // check sum of inflation rights in output equals a64[0]
        test;

        ret;
    };
    Lib::assemble::<Instr<RgbIsa<MemContract>>>(&code)
        .expect("wrong inflatable asset genesis valdiation script")
}

pub(crate) fn ifa_lib_transition() -> Lib {
    let code = rgbasm! {
        // Checking that the sum of inputs is equal to the sum of outputs
        put     a8[0],ERRNO_NON_EQUAL_IN_OUT;  // set errno
        svs     OS_ASSET;  // verify sum
        test;  // check it didn't fail
        ret;  // return execution flow
    };
    Lib::assemble::<Instr<RgbIsa<MemContract>>>(&code).expect("wrong transition validation script")
}

pub(crate) fn ifa_lib_inflation() -> Lib {
    #[allow(clippy::diverging_sub_expression)]
    let code = rgbasm! {
        // Set common offsets
        put     a8[1],0;
        put     a16[0],0;

        // Check reported issued supply equals sum of asset allocations in output
        put     a8[0],ERRNO_ISSUED_MISMATCH;  // set errno
        ldg     GS_ISSUED_SUPPLY,a8[1],s16[0];  // read issued supply global state
        extr    s16[0],a64[0],a16[0];  // and store it in a64[0]
        sas     OS_ASSET;  // check sum of asset allocations in output equals issued_supply
        test;
        cpy     a64[0],a64[1];  // store issued supply in a64[1] for later

        // Check reported allowed inflation equals sum of inflation rights in output
        put     a8[0],ERRNO_INFLATION_MISMATCH;  // set errno
        ldm     MS_ALLOWED_INFLATION,s16[0];  // read allowed inflation global state
        extr    s16[0],a64[0],a16[0];  // and store it in a64[0]
        sas     OS_INFLATION;  // check sum of inflation rights in output equals a64[0]
        test;

        // Check that input inflation rights equals issued supply + allowed inflation
        put     a8[0],ERRNO_INFLATION_EXCEEDS_ALLOWANCE;
        add.uc  a64[1],a64[0];  // result is stored in a64[0]
        test;  // fails in case of an overflow
        sps     OS_INFLATION;  // check sum of inflation rights in input equals a64[0]
        test;

        ret;
    };
    Lib::assemble::<Instr<RgbIsa<MemContract>>>(&code).expect("wrong inflation validation script")
}

fn ifa_standard_types() -> StandardTypes { StandardTypes::with(rgb_contract_stl()) }

fn ifa_schema() -> Schema {
    let types = ifa_standard_types();

    let alu_id_transition = ifa_lib_transition().id();

    Schema {
        ffv: zero!(),
        name: tn!("InflatableFungibleAsset"),
        meta_types: tiny_bmap! {
            MS_ALLOWED_INFLATION => MetaDetails {
                sem_id: types.get("RGBContract.Amount"),
                name: fname!("allowedInflation"),
            }
        },
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
                global_state_schema: GlobalStateSchema::many(types.get("RGBContract.Amount")),
                name: fname!("issuedSupply"),
            },
            GS_MAX_SUPPLY => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.Amount")),
                name: fname!("maxSupply"),
            },
            GS_OPID_REJECT_URL => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.OpidRejectUrl")),
                name: fname!("opidRejectUrl"),
            },
        },
        owned_types: tiny_bmap! {
            OS_ASSET => AssignmentDetails {
                owned_state_schema: OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
                name: fname!("assetOwner"),
                default_transition: TS_TRANSFER,
            },
            OS_INFLATION => AssignmentDetails {
                owned_state_schema: OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
                name: fname!("inflationAllowance"),
                default_transition: TS_INFLATION,
            },
            OS_REPLACE => AssignmentDetails {
                owned_state_schema: OwnedStateSchema::Declarative,
                name: fname!("replaceRight"),
                default_transition: TS_REPLACE,
            }
        },
        genesis: GenesisSchema {
            metadata: none!(),
            globals: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_TERMS => Occurrences::Once,
                GS_ISSUED_SUPPLY => Occurrences::Once,
                GS_MAX_SUPPLY => Occurrences::Once,
                GS_OPID_REJECT_URL => Occurrences::NoneOrOnce,
            },
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::NoneOrMore,
                OS_INFLATION => Occurrences::NoneOrMore,
                OS_REPLACE => Occurrences::NoneOrMore,
            },
            validator: Some(LibSite::with(0, ifa_lib_genesis().id())),
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
                    validator: Some(LibSite::with(0, alu_id_transition))
                },
                name: fname!("transfer"),
            },
            TS_INFLATION => TransitionDetails {
                transition_schema: TransitionSchema {
                    metadata: tiny_bset![MS_ALLOWED_INFLATION],
                    globals: tiny_bmap! {
                        GS_ISSUED_SUPPLY => Occurrences::Once,
                    },
                    inputs: tiny_bmap! {
                        OS_INFLATION => Occurrences::OnceOrMore
                    },
                    assignments: tiny_bmap! {
                        OS_ASSET => Occurrences::NoneOrMore,  // allow moving right alone
                        OS_INFLATION => Occurrences::NoneOrMore
                    },
                    validator: Some(LibSite::with(0, ifa_lib_inflation().id()))
                },
                name: fname!("inflate"),
            },
            TS_BURN => TransitionDetails {
                transition_schema: TransitionSchema {
                    metadata: none!(),
                    globals: none!(),
                    inputs: tiny_bmap! {
                        OS_ASSET => Occurrences::NoneOrMore,
                        OS_REPLACE => Occurrences::NoneOrMore,
                        OS_INFLATION => Occurrences::NoneOrMore,
                    },
                    assignments: none!(),
                    validator: None
                },
                name: fname!("burn"),
            },
            TS_REPLACE => TransitionDetails {
                transition_schema: TransitionSchema {
                    metadata: none!(),
                    globals: none!(),
                    inputs: tiny_bmap! {
                        OS_ASSET => Occurrences::NoneOrMore,  // allow moving right alone
                        OS_REPLACE => Occurrences::OnceOrMore,
                    },
                    assignments: tiny_bmap! {
                        OS_ASSET => Occurrences::NoneOrMore,  // allow moving right alone
                        OS_REPLACE => Occurrences::OnceOrMore,
                    },
                    validator: Some(LibSite::with(0, alu_id_transition))
                },
                name: fname!("replace"),
            },
        },
        default_assignment: Some(OS_ASSET),
    }
}

#[derive(Default)]
pub struct InflatableFungibleAsset;

impl IssuerWrapper for InflatableFungibleAsset {
    type Wrapper<S: ContractStateRead> = IfaWrapper<S>;

    fn schema() -> Schema { ifa_schema() }

    fn types() -> TypeSystem { ifa_standard_types().type_system(ifa_schema()) }

    fn scripts() -> Scripts {
        let alu_lib_genesis = ifa_lib_genesis();
        let alu_id_genesis = alu_lib_genesis.id();

        let alu_lib_transition = ifa_lib_transition();
        let alu_id_transition = alu_lib_transition.id();

        let alu_lib_inflation = ifa_lib_inflation();
        let alu_id_inflation = alu_lib_inflation.id();

        Confined::from_checked(bmap! {
            alu_id_genesis => alu_lib_genesis,
            alu_id_transition => alu_lib_transition,
            alu_id_inflation => alu_lib_inflation,
        })
    }
}
#[derive(Clone, Eq, PartialEq, Debug, From)]
pub struct IfaWrapper<S: ContractStateRead>(ContractData<S>);

impl<S: ContractStateRead> SchemaWrapper<S> for IfaWrapper<S> {
    fn with(data: ContractData<S>) -> Self {
        if data.schema.schema_id() != IFA_SCHEMA_ID {
            panic!("the provided schema is not IFA");
        }
        Self(data)
    }
}

impl<S: ContractStateRead> IfaWrapper<S> {
    pub fn spec(&self) -> AssetSpec {
        let strict_val = &self
            .0
            .global("spec")
            .next()
            .expect("IFA requires global state `spec` to have at least one item");
        AssetSpec::from_strict_val_unchecked(strict_val)
    }

    pub fn contract_terms(&self) -> ContractTerms {
        let strict_val = &self
            .0
            .global("terms")
            .next()
            .expect("IFA requires global state `terms` to have at least one item");
        ContractTerms::from_strict_val_unchecked(strict_val)
    }

    pub fn opid_reject_url(&self) -> Option<OpidRejectUrl> {
        self.0
            .global("opidRejectUrl")
            .next()
            .map(|strict_val| OpidRejectUrl::from_strict_val_unchecked(&strict_val))
    }

    pub fn total_issued_supply(&self) -> Amount {
        self.0
            .global("issuedSupply")
            .map(|amount| Amount::from_strict_val_unchecked(&amount))
            .sum()
    }

    pub fn max_supply(&self) -> Amount {
        self.0
            .global("maxSupply")
            .map(|amount| Amount::from_strict_val_unchecked(&amount))
            .sum()
    }

    pub fn allocations<'c>(
        &'c self,
        filter: impl AssignmentsFilter + 'c,
    ) -> impl Iterator<Item = FungibleAllocation> + 'c {
        self.0.fungible_raw(OS_ASSET, filter).unwrap()
    }

    pub fn inflation_allocations<'c>(
        &'c self,
        filter: impl AssignmentsFilter + 'c,
    ) -> impl Iterator<Item = FungibleAllocation> + 'c {
        self.0.fungible_raw(OS_INFLATION, filter).unwrap()
    }

    pub fn replace_rights<'c>(
        &'c self,
        filter: impl AssignmentsFilter + 'c,
    ) -> impl Iterator<Item = RightsAllocation> + 'c {
        self.0.rights_raw(OS_REPLACE, filter).unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::ifa::ifa_schema;
    use crate::IFA_SCHEMA_ID;

    #[test]
    fn schema_id() {
        let schema_id = ifa_schema().schema_id();
        eprintln!("{:#04x?}", schema_id.to_byte_array());
        assert_eq!(IFA_SCHEMA_ID, schema_id);
    }
}
