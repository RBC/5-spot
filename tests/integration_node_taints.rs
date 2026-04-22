// Copyright (c) 2025 Erick Bourgeois, RBC Capital Markets
// SPDX-License-Identifier: Apache-2.0
//! # Integration test: end-to-end node taint reconcile on a real cluster
//!
//! Scope: exercises [`five_spot::reconcilers::reconcile_node_taints`] against a
//! real kube cluster (typically `kind`). Bypasses CAPI/k0smotron entirely — we
//! pick an already-Ready Node in the cluster and drive the taint reconcile
//! directly, which is the only piece that actually touches the Node API. This
//! mirrors Phase 8 of `~/dev/roadmaps/completed-5spot-user-defined-node-taints.md`:
//! prove the apply/update/shrink/conflict semantics against the real API
//! server without needing SSH-reachable hardware for RemoteMachine.
//!
//! ## Running
//! Requires a reachable cluster and is marked `#[ignore]` so `cargo test`
//! stays hermetic. To run:
//!
//! ```bash
//! kind create cluster
//! cargo test --test integration_node_taints -- --ignored --test-threads=1
//! ```
//!
//! `--test-threads=1` is mandatory — all test cases mutate the same shared
//! Node and would race otherwise. Each case uses a unique taint-key prefix
//! (`integration-test.5spot.local/<case>-…`) so leftover state from one case
//! can never be mistaken for input to another, and the cleanup guard at the
//! end of each case scrubs anything it applied even on panic.

use std::collections::BTreeSet;

use k8s_openapi::api::core::v1::{Node, Taint};
use kube::api::{Patch, PatchParams};
use kube::{Api, Client};
use serde_json::json;

use five_spot::crd::{NodeTaint, TaintEffect};
use five_spot::reconcilers::{
    reconcile_node_taints, NodeTaintReconcileOutcome, ReconcileNodeTaintsInput,
};

/// All test-applied taints use this prefix so cleanup can scrub by key prefix
/// without touching anything else on the Node.
const TEST_KEY_PREFIX: &str = "integration-test.5spot.local/";

/// Connect to the default cluster or skip the test with a clear message.
/// Using `try_default()` means developers without a cluster still get a pass
/// with a visible skip log, not a hard failure.
async fn client_or_skip() -> Option<Client> {
    match Client::try_default().await {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("SKIP: no reachable cluster ({e}); run against kind to exercise this test");
            None
        }
    }
}

/// Pick any Ready Node in the cluster. We don't care which — we only need a
/// real, materialised, Ready Node to stand in for the CAPI-provisioned one.
async fn pick_ready_node(client: &Client) -> Option<String> {
    let nodes: Api<Node> = Api::all(client.clone());
    let list = match nodes.list(&Default::default()).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("SKIP: failed to list nodes ({e})");
            return None;
        }
    };
    for n in list.items {
        let name = match n.metadata.name.clone() {
            Some(n) => n,
            None => continue,
        };
        let ready = n
            .status
            .as_ref()
            .and_then(|s| s.conditions.as_ref())
            .map(|cs| cs.iter().any(|c| c.type_ == "Ready" && c.status == "True"))
            .unwrap_or(false);
        if ready {
            return Some(name);
        }
    }
    eprintln!("SKIP: no Ready node found in cluster");
    None
}

/// Remove every taint whose key starts with [`TEST_KEY_PREFIX`] and clear our
/// ownership annotation. Uses a strategic-merge patch with explicit values so
/// we only touch what we applied — never admin taints.
async fn cleanup(client: &Client, node_name: &str) {
    let nodes: Api<Node> = Api::all(client.clone());
    let Ok(node) = nodes.get(node_name).await else {
        return;
    };
    let keep: Vec<Taint> = node
        .spec
        .as_ref()
        .and_then(|s| s.taints.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|t| !t.key.starts_with(TEST_KEY_PREFIX))
        .collect();
    let patch = json!({
        "metadata": { "annotations": { "5spot.finos.org/applied-taints": null } },
        "spec": { "taints": keep },
    });
    let _ = nodes
        .patch(
            node_name,
            &PatchParams::default(),
            &Patch::Merge(patch.clone()),
        )
        .await;
}

/// Fetch the current set of (key, effect) pairs on the Node. Used for
/// assertions after a reconcile — we care about identity, not ordering, and
/// we filter to test-owned keys so admin taints never leak into expectations.
async fn current_test_taint_identities(
    client: &Client,
    node_name: &str,
) -> BTreeSet<(String, String)> {
    let nodes: Api<Node> = Api::all(client.clone());
    let node = nodes.get(node_name).await.expect("Node GET failed");
    node.spec
        .as_ref()
        .and_then(|s| s.taints.as_ref())
        .map(|ts| {
            ts.iter()
                .filter(|t| t.key.starts_with(TEST_KEY_PREFIX))
                .map(|t| (t.key.clone(), t.effect.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn taint(key_suffix: &str, effect: TaintEffect) -> NodeTaint {
    NodeTaint {
        key: format!("{TEST_KEY_PREFIX}{key_suffix}"),
        value: Some("ci".to_string()),
        effect,
    }
}

/// Apply two taints, then shrink to one, then re-apply — verifies that the
/// controller correctly adds, keeps, and removes entries it owns without
/// touching admin state on the same Node.
#[tokio::test]
#[ignore]
async fn apply_update_shrink_cycle() {
    let Some(client) = client_or_skip().await else {
        return;
    };
    let Some(node_name) = pick_ready_node(&client).await else {
        return;
    };
    // Ensure a clean slate in case a prior run died mid-way.
    cleanup(&client, &node_name).await;

    let t_a = taint("cycle-a", TaintEffect::NoSchedule);
    let t_b = taint("cycle-b", TaintEffect::PreferNoSchedule);

    // Apply both taints.
    let outcome = reconcile_node_taints(
        &client,
        ReconcileNodeTaintsInput {
            node_name: &node_name,
            desired: &[t_a.clone(), t_b.clone()],
            previously_applied: &[],
        },
    )
    .await
    .expect("initial apply failed");
    let applied = match outcome {
        NodeTaintReconcileOutcome::Applied { applied } => applied,
        other => panic!("expected Applied, got {other:?}"),
    };
    assert_eq!(applied.len(), 2, "both taints should be applied");

    let identities = current_test_taint_identities(&client, &node_name).await;
    assert!(
        identities.contains(&(t_a.key.clone(), "NoSchedule".to_string())),
        "a taint missing: {identities:?}"
    );
    assert!(
        identities.contains(&(t_b.key.clone(), "PreferNoSchedule".to_string())),
        "b taint missing: {identities:?}"
    );

    // Shrink desired to a single taint. The controller must remove `b` (which
    // it owns via previously_applied) and keep `a`.
    let outcome = reconcile_node_taints(
        &client,
        ReconcileNodeTaintsInput {
            node_name: &node_name,
            desired: std::slice::from_ref(&t_a),
            previously_applied: &applied,
        },
    )
    .await
    .expect("shrink failed");
    let applied = match outcome {
        NodeTaintReconcileOutcome::Applied { applied } => applied,
        other => panic!("expected Applied on shrink, got {other:?}"),
    };
    assert_eq!(applied.len(), 1, "only a should remain");

    let identities = current_test_taint_identities(&client, &node_name).await;
    assert_eq!(
        identities.len(),
        1,
        "exactly one test-owned taint should remain: {identities:?}"
    );
    assert!(identities.contains(&(t_a.key.clone(), "NoSchedule".to_string())));

    cleanup(&client, &node_name).await;
}

/// When an admin manually adds a taint with the same `(key, effect)` as a
/// desired one we did NOT previously apply, the reconcile must surface
/// `Conflict` and leave the admin taint untouched.
#[tokio::test]
#[ignore]
async fn admin_conflict_is_reported_and_admin_taint_preserved() {
    let Some(client) = client_or_skip().await else {
        return;
    };
    let Some(node_name) = pick_ready_node(&client).await else {
        return;
    };
    cleanup(&client, &node_name).await;

    let desired = taint("conflict", TaintEffect::NoSchedule);

    // Simulate an admin who manually added the same (key, effect) with a
    // different value, *before* the controller ever applied it. Because
    // `previously_applied` is empty, the controller does not own it.
    let nodes: Api<Node> = Api::all(client.clone());
    let existing: Vec<Taint> = nodes
        .get(&node_name)
        .await
        .expect("Node GET failed")
        .spec
        .and_then(|s| s.taints)
        .unwrap_or_default();
    let mut new_taints = existing.clone();
    new_taints.push(Taint {
        key: desired.key.clone(),
        value: Some("admin-value".to_string()),
        effect: "NoSchedule".to_string(),
        time_added: None,
    });
    let patch = json!({ "spec": { "taints": new_taints } });
    nodes
        .patch(&node_name, &PatchParams::default(), &Patch::Merge(patch))
        .await
        .expect("admin taint seed failed");

    let outcome = reconcile_node_taints(
        &client,
        ReconcileNodeTaintsInput {
            node_name: &node_name,
            desired: std::slice::from_ref(&desired),
            previously_applied: &[],
        },
    )
    .await
    .expect("reconcile failed");

    match outcome {
        NodeTaintReconcileOutcome::Conflict { conflicts } => {
            assert_eq!(conflicts.len(), 1, "expected 1 conflict, got {conflicts:?}");
            assert_eq!(conflicts[0].key, desired.key);
        }
        other => panic!("expected Conflict, got {other:?}"),
    }

    // The admin taint must still be on the Node with its original value —
    // the controller must not overwrite it.
    let node = nodes.get(&node_name).await.expect("Node GET failed");
    let admin_taint = node
        .spec
        .and_then(|s| s.taints)
        .unwrap_or_default()
        .into_iter()
        .find(|t| t.key == desired.key)
        .expect("admin taint must still be present");
    assert_eq!(
        admin_taint.value.as_deref(),
        Some("admin-value"),
        "admin taint value must not be overwritten"
    );

    // Clean up the admin-seeded taint manually — cleanup() will strip it via
    // the TEST_KEY_PREFIX filter.
    cleanup(&client, &node_name).await;
}
