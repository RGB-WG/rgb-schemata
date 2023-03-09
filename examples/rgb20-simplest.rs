#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_types;

use aluvm::library::{Lib, LibSite};
use amplify::confinement::Confined;
use amplify::hex::FromHex;
use bp::{Chain, Outpoint, Tx, Txid};
use rgbstd::containers::{BindleContent, ContractBuilder};
use rgbstd::interface::{rgb20, IfaceImpl, NamedType};
use rgbstd::persistence::{Inventory, Stock};
use rgbstd::resolvers::ResolveHeight;
use rgbstd::schema::{
    FungibleType, GenesisSchema, GlobalStateSchema, Occurrences, Schema, Script, StateSchema,
    SubSchema, TransitionSchema,
};
use rgbstd::stl::{ContractText, Nominal, Precision, StandardTypes};
use rgbstd::validation::{ResolveTx, TxResolverError};
use rgbstd::vm::{AluScript, ContractOp, EntryPoint, RgbIsa};
use std::convert::Infallible;
use strict_encoding::StrictDumb;

struct DumbResolver;

impl ResolveTx for DumbResolver {
    fn resolve_tx(&self, _txid: Txid) -> Result<Tx, TxResolverError> {
        Ok(Tx::strict_dumb())
    }
}

impl ResolveHeight for DumbResolver {
    type Error = Infallible;
    fn resolve_height(&mut self, _txid: Txid) -> Result<u32, Self::Error> {
        Ok(0)
    }
}

const GS_NOMINAL: u16 = 0;
const GS_CONTRACT: u16 = 1;
const OS_ASSETS: u16 = 0;
const TS_TRANSFER: u16 = 0;

fn schema() -> SubSchema {
    let types = StandardTypes::new();

    let code = [RgbIsa::Contract(ContractOp::PcVs(OS_ASSETS))];
    let alu_lib = Lib::assemble(&code).unwrap();
    let alu_id = alu_lib.id();

    Schema {
        ffv: zero!(),
        subset_of: None,
        type_system: types.type_system(),
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalStateSchema::once(types.get("RGBContract.Nominal")),
            GS_CONTRACT => GlobalStateSchema::once(types.get("RGBContract.ContractText")),
        },
        owned_types: tiny_bmap! {
            OS_ASSETS => StateSchema::Fungible(FungibleType::Unsigned64Bit),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: None,
            globals: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_CONTRACT => Occurrences::Once,
            },
            assignments: tiny_bmap! {
                OS_ASSETS => Occurrences::OnceOrMore,
            },
            valencies: none!(),
        },
        extensions: none!(),
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionSchema {
                metadata: None,
                globals: none!(),
                inputs: tiny_bmap! {
                    OS_ASSETS => Occurrences::OnceOrMore
                },
                assignments: tiny_bmap! {
                    OS_ASSETS => Occurrences::OnceOrMore
                },
                valencies: none!(),
            }
        },
        script: Script::AluVM(AluScript {
            libs: confined_bmap! { alu_id => alu_lib },
            entry_points: confined_bmap! {
                EntryPoint::ValidateOwnedState(OS_ASSETS) => LibSite::with(0, alu_id)
            },
        }),
    }
}

fn iface_impl() -> IfaceImpl {
    let schema = schema();
    let iface = rgb20();

    IfaceImpl {
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        global_state: tiny_bset! {
            NamedType::with(GS_NOMINAL, tn!("Nominal")),
            NamedType::with(GS_CONTRACT, tn!("ContractText")),
        },
        owned_state: tiny_bset! {
            NamedType::with(OS_ASSETS, tn!("Assets")),
        },
        valencies: none!(),
        transitions: tiny_bset! {
            NamedType::with(TS_TRANSFER, tn!("Transfer")),
        },
        extensions: none!(),
    }
}

#[rustfmt::skip]
fn main() {
    let my_schema = schema();
    let schema_bindle = my_schema.bindle();
    eprintln!("{schema_bindle}");

    let iimpl = iface_impl();
    let iimpl_bindle = iimpl.bindle();
    eprintln!("{iimpl_bindle}");

    let nominal = Nominal::new("TEST", "Test asset", Precision::CentiMicro);
    let contract_text = ContractText::default();
    let beneficiary = Outpoint::new(
        Txid::from_hex("623554ac1dcd15496c105a27042c438921f2a82873579be88e74d7ef559a3d91").unwrap(), 
        0
    );

    let contract = ContractBuilder::with(
        rgb20(),
        schema(),
        iface_impl()
        ).expect("schema fails to implement RGB20 interface")

        .set_chain(Chain::Testnet3)

        .add_global_state("Nominal", nominal)
        .expect("invalid nominal")
        
        .add_global_state("ContractText", contract_text)
        .expect("invalid contract text")

        .add_fungible_state("Assets", beneficiary, 1_000_000_0000_0000)
        .expect("invalid asset amount")

        .issue_contract()
        .expect("contract doesn't fit schema requirements");

    let contract_id = contract.contract_id();
    
    let bindle = contract.bindle();
    eprintln!("{bindle}");

    // Let's create some stock - an in-memory stash and inventory around it:
    let mut stock = Stock::default();
    stock.import_iface(rgb20()).unwrap();
    stock.import_schema(schema()).unwrap();
    stock.import_iface_impl(iface_impl()).unwrap();

    // Noe we verify our contract consignment and add it to the stock
    let verified_contract = bindle.unbindle().validate(&mut DumbResolver).expect("failed contract");
    stock.import_contract(verified_contract, &mut DumbResolver).unwrap();
    
    // Reading contract state through the interface from the stock:
    let contract = stock.contract_iface(contract_id, rgb20().iface_id()).unwrap();
    let nominal = contract.global("Nominal").unwrap();
    let allocations = contract.fungible("Assets").unwrap();
    eprintln!("{}", nominal[0]);
    
    for (txout, amount) in allocations {
        eprintln!("(amount={amount}, txout={txout})");
    }
}
