// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

// Testsuite of fatp integration tests.
pub mod zombienet;

use crate::zombienet::{
	default_zn_scenario_builder, relaychain_rococo_local_network_spec as relay,
	relaychain_rococo_local_network_spec::parachain_asset_hub_network_spec as para, NetworkSpawner,
};
use txtesttool::execution_log::ExecutionLog;
use zombienet::DEFAULT_SEND_FUTURE_AND_READY_TXS_TESTS_TIMEOUT_IN_SECS;

// Test which sends future and ready txs from many accounts
// to an unlimited pool of a parachain collator based on the asset-hub-rococo runtime.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn send_future_and_ready_from_many_accounts_to_parachain() {
	let net = NetworkSpawner::from_toml_with_env_logger(para::HIGH_POOL_LIMIT_FATP)
		.await
		.unwrap();

	// Wait for the parachain collator to start block production.
	net.wait_for_block_production("charlie").await.unwrap();

	// Create future & ready txs executors.
	let ws = net.node_rpc_uri("charlie").unwrap();
	let future_scenario_executor = default_zn_scenario_builder(&net)
		.with_rpc_uri(ws.clone())
		.with_start_id(0)
		.with_last_id(99)
		.with_nonce_from(Some(100))
		.with_txs_count(100)
		.with_executor_id("future-txs-executor".to_string())
		.with_timeout_in_secs(DEFAULT_SEND_FUTURE_AND_READY_TXS_TESTS_TIMEOUT_IN_SECS)
		.build()
		.await;
	let ready_scenario_executor = default_zn_scenario_builder(&net)
		.with_rpc_uri(ws)
		.with_start_id(0)
		.with_last_id(99)
		.with_nonce_from(Some(0))
		.with_txs_count(100)
		.with_executor_id("ready-txs-executor".to_string())
		.with_timeout_in_secs(DEFAULT_SEND_FUTURE_AND_READY_TXS_TESTS_TIMEOUT_IN_SECS)
		.build()
		.await;

	// Execute transactions and fetch the execution logs.
	let (future_logs, ready_logs) = futures::future::join(
		future_scenario_executor.execute(),
		ready_scenario_executor.execute(),
	)
	.await;

	let finalized_future =
		future_logs.values().filter_map(|default_log| default_log.finalized()).count();
	let finalized_ready =
		ready_logs.values().filter_map(|default_log| default_log.finalized()).count();

	assert_eq!(finalized_future, 10_000);
	assert_eq!(finalized_ready, 10_000);
}

// Test which sends future and ready txs from many accounts
// to an unlimited pool of a relaychain node based on `rococo-local` runtime.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn send_future_and_ready_from_many_accounts_to_relaychain() {
	let net = NetworkSpawner::from_toml_with_env_logger(relay::HIGH_POOL_LIMIT_FATP)
		.await
		.unwrap();

	// Wait for the paracha validator to start block production & have its genesis block
	// finalized.
	net.wait_for_block_production("alice").await.unwrap();

	// Create future & ready txs executors.
	let ws = net.node_rpc_uri("alice").unwrap();
	let future_scenario_executor = default_zn_scenario_builder(&net)
		.with_rpc_uri(ws.clone())
		.with_start_id(0)
		.with_last_id(99)
		.with_nonce_from(Some(100))
		.with_txs_count(100)
		.with_executor_id("future-txs-executor".to_string())
		.with_timeout_in_secs(DEFAULT_SEND_FUTURE_AND_READY_TXS_TESTS_TIMEOUT_IN_SECS)
		.build()
		.await;
	let ready_scenario_executor = default_zn_scenario_builder(&net)
		.with_rpc_uri(ws)
		.with_start_id(0)
		.with_last_id(99)
		.with_nonce_from(Some(0))
		.with_txs_count(100)
		.with_executor_id("ready-txs-executor".to_string())
		.with_timeout_in_secs(DEFAULT_SEND_FUTURE_AND_READY_TXS_TESTS_TIMEOUT_IN_SECS)
		.build()
		.await;

	// Execute transactions and fetch the execution logs.
	// Execute transactions and fetch the execution logs.
	let (future_logs, ready_logs) = futures::future::join(
		future_scenario_executor.execute(),
		ready_scenario_executor.execute(),
	)
	.await;

	let finalized_future =
		future_logs.values().filter_map(|default_log| default_log.finalized()).count();
	let finalized_ready =
		ready_logs.values().filter_map(|default_log| default_log.finalized()).count();

	assert_eq!(finalized_future, 10_000);
	assert_eq!(finalized_ready, 10_000);
}

// Test which sends 5m transactions to parachain. Long execution time expected.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn send_5m_from_many_accounts_to_parachain() {
	let net = NetworkSpawner::from_toml_with_env_logger(para::HIGH_POOL_LIMIT_FATP)
		.await
		.unwrap();

	// Wait for the parachain collator to start block production.
	net.wait_for_block_production("charlie").await.unwrap();

	// Create future & ready txs executors.
	let ws = net.node_rpc_uri("charlie").unwrap();
	let executor = default_zn_scenario_builder(&net)
		.with_rpc_uri(ws)
		.with_start_id(0)
		.with_last_id(999)
		.with_txs_count(5_000)
		.with_executor_id("txs-executor".to_string())
		.build()
		.await;

	// Execute transactions and fetch the execution logs.
	let execution_logs = executor.execute().await;
	let finalized_txs = execution_logs.values().filter_map(|tx_log| tx_log.finalized()).count();

	assert_eq!(finalized_txs, 5_000_000);
}

/// Internal test that allows to observe how transcactions are gossiped in the network. Requires
/// external tool to track transactions presence at nodes. Was used to evaluate some metrics of
/// existing transaction protocol.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn gossiping() {
	let net = NetworkSpawner::from_toml_with_env_logger(relay::HIGH_POOL_LIMIT_FATP_TRACE)
		.await
		.unwrap();

	// Wait for the parachain collator to start block production.
	net.wait_for_block_production("a00").await.unwrap();

	// Create the txs executor.
	let ws = net.node_rpc_uri("a00").unwrap();
	let executor = default_zn_scenario_builder(&net)
		.with_rpc_uri(ws)
		.with_start_id(0)
		.with_last_id(999)
		.with_executor_id("txs-executor".to_string())
		.build()
		.await;

	// Execute transactions and fetch the execution logs.
	let execution_logs = executor.execute().await;
	let finalized_txs = execution_logs.values().filter_map(|tx_log| tx_log.finalized()).count();

	assert_eq!(finalized_txs, 1000);

	tracing::info!("BASEDIR: {:?}", net.base_dir_path());
}
