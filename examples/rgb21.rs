use std::convert::Infallible;
use std::fs;

use amplify::confinement::SmallBlob;
use amplify::hex::FromHex;
use amplify::Wrapper;
use bp::{Outpoint, Tx, Txid};
use rgb_schemata::{uda_rgb21, uda_schema};
use rgbstd::containers::BindleContent;
use rgbstd::interface::rgb21::{Allocation, EmbeddedMedia, OwnedFraction, TokenData, TokenIndex};
use rgbstd::interface::{rgb21, ContractBuilder};
use rgbstd::invoice::Precision;
use rgbstd::persistence::{Inventory, Stock};
use rgbstd::resolvers::ResolveHeight;
use rgbstd::stl::{self, DivisibleAssetSpec, RicardianContract, Timestamp};
use rgbstd::validation::{ResolveTx, TxResolverError};
use rgbstd::{Layer1, WitnessAnchor, XAnchor};
use strict_encoding::StrictDumb;

struct DumbResolver;

impl ResolveTx for DumbResolver {
    fn resolve_bp_tx(&self, _: Layer1, _: Txid) -> Result<Tx, TxResolverError> {
        Ok(Tx::strict_dumb())
    }
}

impl ResolveHeight for DumbResolver {
    type Error = Infallible;
    fn resolve_anchor(&mut self, _: &XAnchor) -> Result<WitnessAnchor, Self::Error> {
        Ok(WitnessAnchor::strict_dumb())
    }
}

#[rustfmt::skip]
fn main() {
    let spec = DivisibleAssetSpec::new("TEST", "Test uda", Precision::Indivisible);
    let terms = RicardianContract::default();
    let created = Timestamp::now();
    let beneficiary = Outpoint::new(
        Txid::from_hex("623554ac1dcd15496c105a27042c438921f2a82873579be88e74d7ef559a3d91").unwrap(),
        0
    );

    let fraction = OwnedFraction::from_inner(1);
    let index = TokenIndex::from_inner(2);

    let preview = EmbeddedMedia {
        ty: stl::MediaType::with("text/*"),
        data: SmallBlob::try_from_iter(vec![0, 0]).expect("invalid data"),
    };

    let token_data = TokenData { index, preview: Some(preview), ..Default::default() };

    let allocation = Allocation::with(index, fraction);
    let contract = ContractBuilder::testnet(
        rgb21(),
        uda_schema(),
        uda_rgb21(),
        ).expect("schema fails to implement RGB21 interface")

        .add_global_state("tokens", token_data)
        .expect("invalid token data")

        .add_global_state("spec", spec)
        .expect("invalid nominal")

        .add_global_state("created", created)
        .expect("invalid creation date")

        .add_global_state("terms", terms)
        .expect("invalid contract text")

        .add_data("assetOwner", beneficiary, allocation)
        .expect("invalid asset blob")

        .issue_contract()
        .expect("contract doesn't fit schema requirements");
    eprintln!("{}", serde_yaml::to_string(&contract.genesis).unwrap());

    let contract_id = contract.contract_id();
    debug_assert_eq!(contract_id, contract.contract_id());

    let bindle = contract.bindle();
    eprintln!("{bindle}");
    bindle.save("examples/rgb21-simplest.contract.rgb").expect("unable to save contract");
    fs::write("examples/rgb21-simplest.contract.rgba", bindle.to_string()).expect("unable to save contract");

    // Let's create some stock - an in-memory stash and inventory around it:
    let mut stock = Stock::default();
    stock.import_iface(rgb21()).unwrap();
    stock.import_schema(uda_schema()).unwrap();
    stock.import_iface_impl(uda_rgb21()).unwrap();

    // Noe we verify our contract consignment and add it to the stock
    let verified_contract = match bindle.unbindle().validate(&mut DumbResolver, true) {
        Ok(consignment) => consignment,
        Err(consignment) => {
            panic!("can't produce valid consignment. Report: {}", consignment.validation_status().expect("status always present upon validation"));
        }
    };
    stock.import_contract(verified_contract, &mut DumbResolver).unwrap();

    // Reading contract state through the interface from the stock:
    let contract = stock.contract_iface_id(contract_id, rgb21().iface_id()).unwrap();
    let nominal = contract.global("spec").unwrap();
    eprintln!("{}", nominal[0]);
}
