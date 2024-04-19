use std::fs;

use amplify::confinement::SmallBlob;
use amplify::hex::FromHex;
use amplify::{Bytes, Wrapper};
use bp::Txid;
use ifaces::rgb21::{EmbeddedMedia, TokenData};
use ifaces::{IssuerWrapper, Rgb21};
use rgbstd::containers::{FileContent, Kit};
use rgbstd::invoice::Precision;
use rgbstd::persistence::{MemIndex, MemStash, MemState, Stock};
use rgbstd::stl::{AssetSpec, AssetTerms, Attachment, MediaType, RicardianContract};
use rgbstd::{Allocation, GenesisSeal, TokenIndex, XChain};
use schemata::dumb::DumbResolver;
use schemata::UniqueDigitalAsset;
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
    let terms = AssetTerms {
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
    let allocation = Allocation::with(index, 1);

    // Let's create some stock - an in-memory stash and inventory around it:
    let kit = Kit::load_file("schemata/UniqueDigitalAsset.rgb").unwrap().validate().unwrap();
    let mut stock = Stock::<MemStash, MemState, MemIndex>::default();
    stock.import_kit(kit).expect("invalid issuer kit");

    let contract = stock.contract_builder(
        UniqueDigitalAsset::schema().schema_id(),
        "RGB21Unique",
        ).expect("schema fails to implement RGB21 interface")

        .add_global_state("tokens", token_data)
        .expect("invalid token data")

        .add_global_state("spec", spec)
        .expect("invalid nominal")

        .add_global_state("terms", terms)
        .expect("invalid contract text")

        .add_data("assetOwner", beneficiary, allocation)
        .expect("invalid asset blob")

        .issue_contract()
        .expect("contract doesn't fit schema requirements");

    let contract_id = contract.contract_id();

    eprintln!("{contract}");
    contract.save_file("test/rgb21-example.rgb").expect("unable to save contract");
    contract.save_armored("test/rgb21-example.rgba").expect("unable to save armored contract");

    stock.import_contract(contract, &mut DumbResolver).unwrap();

    // Reading contract state through the interface from the stock:
    let contract = stock.contract_iface_class::<Rgb21>(contract_id).unwrap();
    let contract = Rgb21::from(contract);
    eprintln!("{}", serde_json::to_string(&contract.spec()).unwrap());
}
