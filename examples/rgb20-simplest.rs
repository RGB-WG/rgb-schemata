#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_types;
#[macro_use]
extern crate serde_json;

use aluvm::library::{Lib, LibSite};
use amplify::confinement::Confined;
use rgb::containers::BindleContent;
use rgb::interface::{rgb20, IfaceImpl, NamedType};
use rgb::schema::{
    FungibleType, GenesisSchema, GlobalStateSchema, Occurrences, Schema, Script, StateSchema,
    SubSchema, TransitionSchema,
};
use rgb::stl::{ContractText, Nominal, StandardTypes};
use rgb::vm::{AluScript, ContractOp, EntryPoint, RgbIsa};
use strict_types::encoding::libname;
use strict_types::typelib::build::LibBuilder;
use strict_types::typesys::SystemBuilder;

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

fn iface() -> IfaceImpl {
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

fn main() {
    let schema = schema();
    let schema_bindle = schema.bindle();
    eprintln!("{schema_bindle}");

    let iface = iface();
    let iface_bindle = iface.bindle();
    eprintln!("{iface_bindle}");

    /*
    let forge = Forge::with(schema, interface).unwrap();

    let contract = forge.issue(
        json!({
            "Nominal": {
                "Ticker": "TEST",
                "Name": "Test asset",
                "Precision": 8
            },
            "Contract": "",
        }),
        json!({
            "Assets": [
                "623554ac1dcd15496c105a27042c438921f2a82873579be88e74d7ef559a3d91:0": 1_000_000__0000_0000
            ]
        }),
    ).unwrap();

    eprintln!("{contract:X}");

    eprintln!("{}", contract.state());
     */
}
