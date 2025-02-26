// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::*;

use move_cli::base::reroot_path;

use move_package::BuildConfig;

use moveos_verifier::build::run_verifier;

use crate::types::WalletContextOptions;
use std::{collections::BTreeMap, path::PathBuf};

/// Build the package at `path`. If no path is provided defaults to current directory.
#[derive(Parser)]
#[clap(name = "build")]
pub struct Build {
    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=0x5678
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, parse(try_from_str = crate::utils::parse_map), default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, String>,

    #[clap(flatten)]
    config_options: WalletContextOptions,
}

impl Build {
    pub async fn execute(self, path: Option<PathBuf>, config: BuildConfig) -> anyhow::Result<()> {
        let context = self.config_options.build().await?;

        let mut config = config;
        config.additional_named_addresses = context.parse_account_args(self.named_addresses)?;

        let additional_named_address = config.additional_named_addresses.clone();

        let rerooted_path = reroot_path(path)?;
        if config.fetch_deps_only {
            if config.test_mode {
                config.dev_mode = true;
            }
            config.download_deps_for_package(&rerooted_path, &mut std::io::stdout())?;
            return Ok(());
        }

        let mut package = config.compile_package_no_exit(&rerooted_path, &mut std::io::stdout())?;

        run_verifier(rerooted_path, additional_named_address, &mut package);

        Ok(())
    }
}
