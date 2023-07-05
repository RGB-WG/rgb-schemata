// RGB schemata by LNP/BP Standards Association
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2023 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2023 LNP/BP Standards Association. All rights reserved.
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

use std::{fs, io};

use rgb_schemata::{cfa_rgb25, cfa_schema, nia_rgb20, nia_schema, uda_rgb21, uda_schema};
use rgbstd::containers::BindleContent;
use rgbstd::interface::{rgb20, rgb21, rgb25};

fn main() -> io::Result<()> {
    let rgb20_bindle = rgb20().bindle();
    rgb20_bindle.save("interfaces/RGB20.rgb")?;
    fs::write("interfaces/RGB20.rgba", rgb20_bindle.to_string())?;

    let rgb21_bindle = rgb21().bindle();
    rgb21_bindle.save("interfaces/RGB21.rgb")?;
    fs::write("interfaces/RGB21.rgba", rgb21_bindle.to_string())?;

    let rgb25_bindle = rgb25().bindle();
    rgb25_bindle.save("interfaces/RGB25.rgb")?;
    fs::write("interfaces/RGB25.rgba", rgb25_bindle.to_string())?;

    nia()?;
    uda()?;
    cfa()?;

    Ok(())
}

fn nia() -> io::Result<()> {
    let schema_bindle = nia_schema().bindle();
    schema_bindle.save("schemata/NonInflatableAssets.rgb")?;
    fs::write("schemata/NonInflatableAssets.rgba", schema_bindle.to_string())?;

    let iimpl_bindle = nia_rgb20().bindle();
    iimpl_bindle.save("schemata/NonInflatableAssets-RGB20.rgb")?;
    fs::write("schemata/NonInflatableAssets-RGB20.rgba", iimpl_bindle.to_string())?;

    Ok(())
}

fn uda() -> io::Result<()> {
    let schema_bindle = uda_schema().bindle();
    schema_bindle.save("schemata/UniqueDigitalAsset.rgb")?;
    fs::write("schemata/UniqueDigitalAsset.rgba", schema_bindle.to_string())?;

    let iimpl_bindle = uda_rgb21().bindle();
    iimpl_bindle.save("schemata/UniqueDigitalAsset-RGB21.rgb")?;
    fs::write("schemata/UniqueDigitalAsset-RGB21.rgba", iimpl_bindle.to_string())?;

    Ok(())
}

fn cfa() -> io::Result<()> {
    let schema_bindle = cfa_schema().bindle();
    schema_bindle.save("schemata/CollectibleFungibleAssets.rgb")?;
    fs::write("schemata/CollectibleFungibleAssets.rgba", schema_bindle.to_string())?;

    let iimpl_bindle = cfa_rgb25().bindle();
    iimpl_bindle.save("schemata/CollectibleFungibleAssets-RGB25.rgb")?;
    fs::write("schemata/CollectibleFungibleAssets-RGB25.rgba", iimpl_bindle.to_string())?;

    Ok(())
}
