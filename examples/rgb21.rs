use std::fs;

use amplify::confinement::SmallBlob;
use amplify::hex::FromHex;
use amplify::{Bytes, Wrapper};
use bp::Txid;
use ifaces::rgb21::{EmbeddedMedia, NftAllocation, TokenData, TokenIndex};
use ifaces::stl::*;
use ifaces::{IssuerWrapper, Rgb21};
use rgbstd::containers::{ConsignmentExt, FileContent, Kit};
use rgbstd::interface::{FilterIncludeAll, Output};
use rgbstd::persistence::Stock;
use rgbstd::{GenesisSeal, XChain, XWitnessId};
use schemata::UniqueDigitalAsset;
use schemata::dumb::NoResolver;
use sha2::{Digest, Sha256};

#[rustfmt::skip]
fn main() {
    let spec = AssetSpec::new("TEST", "Test uda", Precision::Indivisible);
    let beneficiary_txid =
        Txid::from_hex("14295d5bb1a191cdb6286dc0944df938421e3dfcbf0811353ccac4100c2068c5").unwrap();
    let beneficiary = XChain::Bitcoin(GenesisSeal::tapret_first_rand(beneficiary_txid, 1));

    let index = TokenIndex::from_inner(2);

    let file_bytes = fs::read("README.md").unwrap();
    let mut hasher = Sha256::new();
    hasher.update(file_bytes);
    let file_hash = hasher.finalize();
    let terms = ContractTerms {
        text: RicardianContract::default(),
        media: Some(Attachment {
            ty: MediaType::with("text/*"),
            digest: Bytes::from_byte_array(file_hash),
        }),
    };
    let preview = EmbeddedMedia {
        ty: MediaType::with("image/*"),
        data: SmallBlob::try_from_iter(vec![0, 0]).expect("invalid data"),
    };

    let token_data = TokenData { index, preview: Some(preview), ..Default::default() };
    let allocation = NftAllocation::with(index, 1);

    // Let's create some stock - an in-memory stash and inventory around it:
    let kit = Kit::load_file("schemata/UniqueDigitalAsset.rgb").unwrap().validate().unwrap();
    let mut stock = Stock::in_memory();
    stock.import_kit(kit).expect("invalid issuer kit");

    let contract = stock.contract_builder("ssi:anonymous",
        UniqueDigitalAsset::schema().schema_id(),
        "RGB21Unique",
        ).expect("schema fails to implement RGB21 interface")

        .serialize_global_state("tokens", &token_data)
        .expect("invalid token data")

        .serialize_global_state("spec", &spec)
        .expect("invalid nominal")

        .serialize_global_state("terms", &terms)
        .expect("invalid contract text")

        .serialize_owned_state("assetOwner", beneficiary, &allocation, None)
        .expect("invalid asset blob")

        .issue_contract()
        .expect("contract doesn't fit schema requirements");

    let contract_id = contract.contract_id();

    eprintln!("{contract}");
    contract.save_file("test/rgb21-example.rgb").expect("unable to save contract");
    contract.save_armored("test/rgb21-example.rgba").expect("unable to save armored contract");

    stock.import_contract(contract, NoResolver).unwrap();

    // Reading contract state through the interface from the stock:
    let contract = stock.contract_iface_class::<Rgb21>(contract_id).unwrap();
    let allocations = contract.allocations(&FilterIncludeAll);

    eprintln!("{}", contract.spec());
    for Output  { seal, state, witness, .. } in allocations {
        let witness = witness.as_ref().map(XWitnessId::to_string).unwrap_or("~".to_owned());
        eprintln!("state ({state}), owner {seal}, witness {witness}");
    }
}
