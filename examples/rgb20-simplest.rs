#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_types;

use amplify::ascii::AsciiString;
use amplify::confinement::{Confined, SmallString};
use rgb::vm::AluScript;
use rgb::{
    FungibleType, GenesisSchema, GlobalStateSchema, Occurrences, Schema, Script, StateSchema,
    TransitionSchema,
};
use strict_encoding::StrictDumb;
use strict_types::encoding::libname;
use strict_types::typelib::build::LibBuilder;
use strict_types::typesys::SystemBuilder;

const LIB_NAME: &str = "RGBStd";

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
#[repr(u8)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME, tags = repr, into_u8, try_from_u8)]
pub enum Precision {
    P0 = 0,
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
    #[default]
    P8,
    P9,
    P10,
    P11,
    P12,
    P13,
    P14,
    P15,
    P16,
    P17,
    P18,
}

#[derive(Wrapper, Clone, Eq, PartialEq, Debug, From, StrictType, StrictEncode, StrictDecode)]
#[wrapper(Deref)]
#[strict_type(lib = LIB_NAME)]
pub struct Details(Confined<String, 40, 255>);

impl StrictDumb for Details {
    fn strict_dumb() -> Self {
        Details(
            Confined::try_from(s!("Dumb long description which is stupid and so on...")).unwrap(),
        )
    }
}

#[derive(Clone, Eq, PartialEq, Debug, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME)]
pub struct Nominal {
    ticker: Confined<AsciiString, 1, 8>,
    name: Confined<AsciiString, 1, 40>,
    details: Option<Details>,
    precision: Precision,
}

impl StrictDumb for Nominal {
    fn strict_dumb() -> Self {
        Self::new("DUMB", "Dumb")
    }
}

impl Nominal {
    pub fn new(ticker: &str, name: &str) -> Nominal {
        Nominal {
            ticker: Confined::try_from(AsciiString::from_ascii(ticker).unwrap())
                .expect("invalid ticker"),
            name: Confined::try_from(AsciiString::from_ascii(name).unwrap()).expect("invalid name"),
            details: None,
            precision: Precision::default(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME)]
pub struct Contract(SmallString);

const GS_NOMINAL: u16 = 0;
const GS_CONTRACT: u16 = 1;
const OS_ASSETS: u16 = 0;
const TS_TRANSFER: u16 = 0;

fn schema() -> Schema {
    let lib = LibBuilder::new(libname!(LIB_NAME))
        .process::<Nominal>()
        .expect("failed type definition")
        .process::<Contract>()
        .expect("failed type definition")
        .compile(none!())
        .unwrap();
    let sys = SystemBuilder::new()
        .import(lib)
        .expect("broken lib")
        .finalize()
        .expect("incomplete imports");

    Schema {
        ffv: none!(),
        subset_of: None,
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalStateSchema::once(sys.id_by_name("RGBStd.Nominal").unwrap()),
            GS_CONTRACT => GlobalStateSchema::once(sys.id_by_name("RGBStd.Contract").unwrap()),
        },
        owned_types: tiny_bmap! {
            OS_ASSETS => StateSchema::Fungible(FungibleType::Unsigned64Bit),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: None,
            global_state: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_CONTRACT => Occurrences::Once,
            },
            owned_state: tiny_bmap! {
                OS_ASSETS => Occurrences::OnceOrMore,
            },
            valencies: none!(),
        },
        extensions: none!(),
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionSchema {
                metadata: None,
                global_state: none!(),
                closes: tiny_bmap! {
                    OS_ASSETS => Occurrences::OnceOrMore
                },
                owned_state: tiny_bmap! {
                    OS_ASSETS => Occurrences::OnceOrMore
                },
                valencies: none!(),
            }
        },
        type_system: sys,
        script: Script::AluVM(AluScript::default()),
    }
}

//fn iface() -> IfaceImpl {}

fn main() {
    let schema = schema();
    eprintln!("{schema}")
}
