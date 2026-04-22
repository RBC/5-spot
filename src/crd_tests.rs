// Copyright (c) 2025 Erick Bourgeois, RBC Capital Markets
// SPDX-License-Identifier: Apache-2.0
#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::*;
    use std::collections::HashSet;

    // ========================================================================
    // Day range parsing tests
    // ========================================================================

    #[test]
    fn test_parse_single_day() {
        let days = vec!["mon".to_string()];
        let result = parse_day_ranges(&days).unwrap();
        assert_eq!(result, HashSet::from([0]));
    }

    #[test]
    fn test_parse_day_range() {
        let days = vec!["mon-fri".to_string()];
        let result = parse_day_ranges(&days).unwrap();
        assert_eq!(result, HashSet::from([0, 1, 2, 3, 4]));
    }

    #[test]
    fn test_parse_day_range_wrapping() {
        let days = vec!["fri-mon".to_string()];
        let result = parse_day_ranges(&days).unwrap();
        assert_eq!(result, HashSet::from([0, 4, 5, 6]));
    }

    #[test]
    fn test_parse_day_combinations() {
        let days = vec!["mon-wed,fri-sun".to_string()];
        let result = parse_day_ranges(&days).unwrap();
        assert_eq!(result, HashSet::from([0, 1, 2, 4, 5, 6]));
    }

    #[test]
    fn test_parse_multiple_day_specs() {
        let days = vec!["mon".to_string(), "wed".to_string(), "fri".to_string()];
        let result = parse_day_ranges(&days).unwrap();
        assert_eq!(result, HashSet::from([0, 2, 4]));
    }

    #[test]
    fn test_parse_invalid_day() {
        let days = vec!["monday".to_string()];
        let result = parse_day_ranges(&days);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid day"));
    }

    #[test]
    fn test_parse_invalid_day_range() {
        let days = vec!["mon-tuesday".to_string()];
        let result = parse_day_ranges(&days);
        assert!(result.is_err());
    }

    // ========================================================================
    // Hour range parsing tests
    // ========================================================================

    #[test]
    fn test_parse_single_hour() {
        let hours = vec!["9".to_string()];
        let result = parse_hour_ranges(&hours).unwrap();
        assert_eq!(result, HashSet::from([9]));
    }

    #[test]
    fn test_parse_hour_range() {
        let hours = vec!["9-17".to_string()];
        let result = parse_hour_ranges(&hours).unwrap();
        let expected: HashSet<u8> = (9..=17).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_hour_range_wrapping() {
        let hours = vec!["22-6".to_string()];
        let result = parse_hour_ranges(&hours).unwrap();
        let expected: HashSet<u8> = (22..=23).chain(0..=6).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_hour_combinations() {
        let hours = vec!["0-9,18-23".to_string()];
        let result = parse_hour_ranges(&hours).unwrap();
        let expected: HashSet<u8> = (0..=9).chain(18..=23).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_multiple_hour_specs() {
        let hours = vec!["8".to_string(), "12".to_string(), "18".to_string()];
        let result = parse_hour_ranges(&hours).unwrap();
        assert_eq!(result, HashSet::from([8, 12, 18]));
    }

    #[test]
    fn test_parse_invalid_hour() {
        let hours = vec!["25".to_string()];
        let result = parse_hour_ranges(&hours);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be 0-23"));
    }

    #[test]
    fn test_parse_invalid_hour_range() {
        let hours = vec!["9-25".to_string()];
        let result = parse_hour_ranges(&hours);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_zero_hour() {
        let hours = vec!["0".to_string()];
        let result = parse_hour_ranges(&hours).unwrap();
        assert_eq!(result, HashSet::from([0]));
    }

    #[test]
    fn test_parse_max_hour() {
        let hours = vec!["23".to_string()];
        let result = parse_hour_ranges(&hours).unwrap();
        assert_eq!(result, HashSet::from([23]));
    }

    // ========================================================================
    // ScheduleSpec tests
    // ========================================================================

    #[test]
    fn test_schedule_spec_get_active_weekdays() {
        let spec = ScheduleSpec {
            days_of_week: vec!["mon-fri".to_string()],
            hours_of_day: vec!["9-17".to_string()],
            timezone: "UTC".to_string(),
            enabled: true,
        };

        let weekdays = spec.get_active_weekdays().unwrap();
        assert_eq!(weekdays, Some(HashSet::from([0, 1, 2, 3, 4])));
    }

    #[test]
    fn test_schedule_spec_get_active_hours() {
        let spec = ScheduleSpec {
            days_of_week: vec!["mon-fri".to_string()],
            hours_of_day: vec!["9-17".to_string()],
            timezone: "UTC".to_string(),
            enabled: true,
        };

        let hours = spec.get_active_hours().unwrap();
        let expected: HashSet<u8> = (9..=17).collect();
        assert_eq!(hours, Some(expected));
    }

    // ========================================================================
    // Condition tests
    // ========================================================================

    #[test]
    fn test_condition_creation() {
        let condition = Condition::new(
            "Ready",
            "True",
            "ReconcileSucceeded",
            "Resource reconciled successfully",
        );

        assert_eq!(condition.r#type, "Ready");
        assert_eq!(condition.status, "True");
        assert_eq!(condition.reason, "ReconcileSucceeded");
        assert_eq!(condition.message, "Resource reconciled successfully");
        assert!(!condition.last_transition_time.is_empty());
    }

    // ========================================================================
    // Phase string constants tests
    // ========================================================================

    #[test]
    fn test_phase_constants() {
        use crate::constants::*;
        assert_eq!(PHASE_PENDING, "Pending");
        assert_eq!(PHASE_ACTIVE, "Active");
        assert_eq!(PHASE_INACTIVE, "Inactive");
        assert_eq!(PHASE_SHUTTING_DOWN, "ShuttingDown");
        assert_eq!(PHASE_DISABLED, "Disabled");
        assert_eq!(PHASE_TERMINATED, "Terminated");
        assert_eq!(PHASE_ERROR, "Error");
        assert_eq!(PHASE_EMERGENCY_REMOVE, "EmergencyRemove");
    }

    #[test]
    fn test_reason_emergency_reclaim_disabled_schedule_is_camelcase() {
        use crate::constants::REASON_EMERGENCY_RECLAIM_DISABLED_SCHEDULE;
        assert_eq!(
            REASON_EMERGENCY_RECLAIM_DISABLED_SCHEDULE,
            "EmergencyReclaimDisabledSchedule"
        );
    }

    #[test]
    fn test_emergency_drain_timeout_bounded() {
        use crate::constants::{EMERGENCY_DRAIN_TIMEOUT_SECS, MAX_DURATION_SECS};
        // const block so the assertion is resolved at compile time —
        // guards against a future refactor that sets the timeout to 0
        // or overflows past the 24h cap.
        const _: () = assert!(
            EMERGENCY_DRAIN_TIMEOUT_SECS > 0 && EMERGENCY_DRAIN_TIMEOUT_SECS <= MAX_DURATION_SECS,
            "EMERGENCY_DRAIN_TIMEOUT_SECS must be within (0, MAX_DURATION_SECS]"
        );
    }

    // ========================================================================
    // Emergency reclaim annotation / label constants (roadmap Phase 1 / 2.5)
    // ========================================================================

    #[test]
    fn test_reclaim_annotation_constants_under_5spot_namespace() {
        use crate::constants::*;
        assert_eq!(
            RECLAIM_REQUESTED_ANNOTATION,
            "5spot.finos.org/reclaim-requested"
        );
        assert_eq!(RECLAIM_REASON_ANNOTATION, "5spot.finos.org/reclaim-reason");
        assert_eq!(
            RECLAIM_REQUESTED_AT_ANNOTATION,
            "5spot.finos.org/reclaim-requested-at"
        );
        assert_eq!(RECLAIM_REQUESTED_VALUE, "true");
    }

    #[test]
    fn test_reclaim_agent_label_constants() {
        use crate::constants::*;
        assert_eq!(RECLAIM_AGENT_LABEL, "5spot.finos.org/reclaim-agent");
        assert_eq!(RECLAIM_AGENT_LABEL_ENABLED, "enabled");
    }

    #[test]
    fn test_reclaim_agent_configmap_and_namespace() {
        use crate::constants::*;
        assert_eq!(RECLAIM_AGENT_NAMESPACE, "5spot-system");
        assert_eq!(RECLAIM_AGENT_CONFIGMAP_PREFIX, "reclaim-agent-");
    }

    #[test]
    fn test_reason_emergency_reclaim_is_camelcase() {
        use crate::constants::REASON_EMERGENCY_RECLAIM;
        assert_eq!(REASON_EMERGENCY_RECLAIM, "EmergencyReclaim");
    }

    #[test]
    fn test_reclaim_annotations_covered_by_reserved_prefixes() {
        // Reserved prefixes on user-supplied labels/annotations must include
        // 5spot.finos.org/ so operators can't inject these keys via the
        // ScheduledMachine.spec.machineTemplate surface.
        use crate::constants::{
            RECLAIM_AGENT_LABEL, RECLAIM_REASON_ANNOTATION, RECLAIM_REQUESTED_ANNOTATION,
            RECLAIM_REQUESTED_AT_ANNOTATION, RESERVED_LABEL_PREFIXES,
        };
        for key in [
            RECLAIM_REQUESTED_ANNOTATION,
            RECLAIM_REASON_ANNOTATION,
            RECLAIM_REQUESTED_AT_ANNOTATION,
            RECLAIM_AGENT_LABEL,
        ] {
            assert!(
                RESERVED_LABEL_PREFIXES.iter().any(|p| key.starts_with(p)),
                "{key} must be covered by a RESERVED_LABEL_PREFIXES entry"
            );
        }
    }

    // ========================================================================
    // Serialization tests
    // ========================================================================

    #[test]
    fn test_scheduled_machine_spec_serialization() {
        use serde_json::json;

        let spec = ScheduledMachineSpec {
            schedule: ScheduleSpec {
                days_of_week: vec!["mon-fri".to_string()],
                hours_of_day: vec!["9-17".to_string()],
                timezone: "UTC".to_string(),
                enabled: true,
            },
            cluster_name: "test-cluster".to_string(),
            bootstrap_spec: EmbeddedResource(json!({
                "apiVersion": "bootstrap.cluster.x-k8s.io/v1beta1",
                "kind": "K0sWorkerConfig",
                "spec": {"args": []}
            })),
            infrastructure_spec: EmbeddedResource(json!({
                "apiVersion": "infrastructure.cluster.x-k8s.io/v1beta1",
                "kind": "RemoteMachine",
                "spec": {"address": "192.168.1.100", "port": 22}
            })),
            machine_template: None,
            priority: 50,
            graceful_shutdown_timeout: "5m".to_string(),
            node_drain_timeout: "5m".to_string(),
            kill_switch: false,
            node_taints: vec![],
            kill_if_commands: None,
        };

        // Test that it serializes without errors
        let json_output = serde_json::to_string(&spec).unwrap();
        assert!(json_output.contains("mon-fri"));
        assert!(json_output.contains("192.168.1.100"));
        assert!(json_output.contains("bootstrap"));
    }

    #[test]
    fn test_scheduled_machine_status_default() {
        let status = ScheduledMachineStatus::default();
        assert_eq!(status.phase, None);
        assert!(status.conditions.is_empty());
        assert_eq!(status.observed_generation, None);
        assert!(!status.in_schedule);
    }

    // ========================================================================
    // Condition.status schema — P2-7 enum constraint tests (TDD)
    // ========================================================================

    fn condition_schema_json() -> serde_json::Value {
        let schema = schemars::schema_for!(Condition);
        serde_json::to_value(schema).expect("schema should serialise")
    }

    // ---- Positive: valid enum values are present in the schema ----

    #[test]
    fn test_condition_status_schema_has_enum_constraint() {
        let schema = condition_schema_json();
        // Navigate to properties.status.enum
        let enum_vals = schema
            .pointer("/definitions/Condition/properties/status/enum")
            .or_else(|| schema.pointer("/properties/status/enum"))
            .expect("Condition.status schema must have an 'enum' constraint for NIST CM-5");
        let arr = enum_vals.as_array().expect("enum must be an array");
        assert_eq!(
            arr.len(),
            3,
            "exactly 3 enum values expected: True, False, Unknown"
        );
    }

    #[test]
    fn test_condition_status_schema_contains_true() {
        let schema = condition_schema_json();
        let enum_vals = schema
            .pointer("/definitions/Condition/properties/status/enum")
            .or_else(|| schema.pointer("/properties/status/enum"))
            .expect("enum must exist");
        assert!(
            enum_vals
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("True")),
            "enum must contain 'True'"
        );
    }

    #[test]
    fn test_condition_status_schema_contains_false() {
        let schema = condition_schema_json();
        let enum_vals = schema
            .pointer("/definitions/Condition/properties/status/enum")
            .or_else(|| schema.pointer("/properties/status/enum"))
            .expect("enum must exist");
        assert!(
            enum_vals
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("False")),
            "enum must contain 'False'"
        );
    }

    #[test]
    fn test_condition_status_schema_contains_unknown() {
        let schema = condition_schema_json();
        let enum_vals = schema
            .pointer("/definitions/Condition/properties/status/enum")
            .or_else(|| schema.pointer("/properties/status/enum"))
            .expect("enum must exist");
        assert!(
            enum_vals
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("Unknown")),
            "enum must contain 'Unknown'"
        );
    }

    // ---- Negative: the Condition type itself still works as a plain String ----

    #[test]
    fn test_condition_new_still_accepts_string_status() {
        // Runtime behaviour unchanged — only the CRD schema gains the constraint
        let c = Condition::new("Ready", "True", "ReconcileSucceeded", "ok");
        assert_eq!(c.status, "True");
    }

    // ========================================================================
    // Status enrichment — providerID + full NodeRef (roadmap Phase 1, TDD RED)
    // ========================================================================

    #[test]
    fn test_status_deserializes_provider_id() {
        let json = serde_json::json!({
            "providerID": "libvirt:///uuid-abc-123",
        });
        let status: ScheduledMachineStatus =
            serde_json::from_value(json).expect("status with providerID must deserialize");
        assert_eq!(
            status.provider_id.as_deref(),
            Some("libvirt:///uuid-abc-123"),
            "providerID must round-trip into ScheduledMachineStatus.provider_id"
        );
    }

    #[test]
    fn test_status_provider_id_omitted_when_none() {
        let status = ScheduledMachineStatus::default();
        let json = serde_json::to_value(&status).expect("serialize default status");
        assert!(
            json.get("providerID").is_none(),
            "providerID must be omitted when None (skip_serializing_if)"
        );
    }

    #[test]
    fn test_status_deserializes_full_node_ref() {
        let json = serde_json::json!({
            "nodeRef": {
                "apiVersion": "v1",
                "kind": "Node",
                "name": "worker-01",
                "uid": "11111111-2222-3333-4444-555555555555",
            }
        });
        let status: ScheduledMachineStatus =
            serde_json::from_value(json).expect("status with full nodeRef must deserialize");
        let node_ref = status.node_ref.expect("nodeRef must be present");
        assert_eq!(node_ref.api_version, "v1");
        assert_eq!(node_ref.kind, "Node");
        assert_eq!(node_ref.name, "worker-01");
        assert_eq!(
            node_ref.uid.as_deref(),
            Some("11111111-2222-3333-4444-555555555555")
        );
    }

    #[test]
    fn test_status_node_ref_uid_optional() {
        let json = serde_json::json!({
            "nodeRef": {
                "apiVersion": "v1",
                "kind": "Node",
                "name": "worker-02",
            }
        });
        let status: ScheduledMachineStatus =
            serde_json::from_value(json).expect("nodeRef without uid must still deserialize");
        let node_ref = status.node_ref.expect("nodeRef must be present");
        assert_eq!(node_ref.name, "worker-02");
        assert!(node_ref.uid.is_none());
    }

    #[test]
    fn test_status_rejects_old_shape_node_ref() {
        // Old shape was LocalObjectReference { name }. Deserializing that into
        // the new NodeRef struct must fail loudly so operators see the migration
        // requirement — silent data loss is unacceptable.
        let json = serde_json::json!({
            "nodeRef": { "name": "worker-legacy" }
        });
        let err = serde_json::from_value::<ScheduledMachineStatus>(json)
            .expect_err("old-shape nodeRef must NOT silently succeed");
        let msg = err.to_string();
        assert!(
            msg.contains("apiVersion") || msg.contains("kind"),
            "error must name a missing field so operators know what changed, got: {msg}"
        );
    }

    // ========================================================================
    // killIfCommands — emergency reclaim opt-in (roadmap Phase 2.5, TDD RED)
    // ========================================================================

    fn base_spec() -> ScheduledMachineSpec {
        use serde_json::json;
        ScheduledMachineSpec {
            schedule: ScheduleSpec {
                days_of_week: vec!["mon-fri".to_string()],
                hours_of_day: vec!["9-17".to_string()],
                timezone: "UTC".to_string(),
                enabled: true,
            },
            cluster_name: "test-cluster".to_string(),
            bootstrap_spec: EmbeddedResource(json!({
                "apiVersion": "bootstrap.cluster.x-k8s.io/v1beta1",
                "kind": "K0sWorkerConfig",
                "spec": {}
            })),
            infrastructure_spec: EmbeddedResource(json!({
                "apiVersion": "infrastructure.cluster.x-k8s.io/v1beta1",
                "kind": "RemoteMachine",
                "spec": {"address": "192.168.1.100", "port": 22}
            })),
            machine_template: None,
            priority: 50,
            graceful_shutdown_timeout: "5m".to_string(),
            node_drain_timeout: "5m".to_string(),
            kill_switch: false,
            node_taints: vec![],
            kill_if_commands: None,
        }
    }

    #[test]
    fn test_kill_if_commands_absent_deserializes_as_none() {
        let json = serde_json::json!({
            "schedule": {
                "daysOfWeek": ["mon-fri"],
                "hoursOfDay": ["9-17"],
                "timezone": "UTC",
                "enabled": true
            },
            "clusterName": "c",
            "bootstrapSpec": {
                "apiVersion": "bootstrap.cluster.x-k8s.io/v1beta1",
                "kind": "K0sWorkerConfig",
                "spec": {}
            },
            "infrastructureSpec": {
                "apiVersion": "infrastructure.cluster.x-k8s.io/v1beta1",
                "kind": "RemoteMachine",
                "spec": {"address": "10.0.0.1", "port": 22}
            }
        });
        let spec: ScheduledMachineSpec =
            serde_json::from_value(json).expect("spec without killIfCommands must deserialize");
        assert!(
            spec.kill_if_commands.is_none(),
            "absent killIfCommands must be None so no agent is installed"
        );
    }

    #[test]
    fn test_kill_if_commands_omitted_from_serialized_output_when_none() {
        let spec = base_spec();
        let json = serde_json::to_value(&spec).expect("serialize spec");
        assert!(
            json.get("killIfCommands").is_none(),
            "killIfCommands must be omitted when None (skip_serializing_if)"
        );
    }

    #[test]
    fn test_kill_if_commands_non_empty_round_trips() {
        let mut spec = base_spec();
        spec.kill_if_commands = Some(vec![
            "java".to_string(),
            "idea".to_string(),
            "steam".to_string(),
        ]);
        let json = serde_json::to_value(&spec).expect("serialize");
        assert_eq!(
            json["killIfCommands"],
            serde_json::json!(["java", "idea", "steam"]),
            "non-empty list must serialize as camelCase killIfCommands"
        );
        let round: ScheduledMachineSpec = serde_json::from_value(json).expect("round-trip");
        assert_eq!(
            round.kill_if_commands.as_deref(),
            Some(["java".to_string(), "idea".to_string(), "steam".to_string()].as_slice())
        );
    }

    #[test]
    fn test_kill_if_commands_empty_list_deserializes_as_some_empty() {
        // Empty list is a valid but meaningless configuration. Preserve the
        // distinction between "absent" (no opt-in) and "present but empty" so
        // the controller can surface a condition warning on empty lists rather
        // than silently treating them as opt-out.
        let json = serde_json::json!({
            "schedule": {"daysOfWeek": [], "hoursOfDay": [], "timezone": "UTC", "enabled": true},
            "clusterName": "c",
            "bootstrapSpec": {
                "apiVersion": "bootstrap.cluster.x-k8s.io/v1beta1",
                "kind": "K0sWorkerConfig",
                "spec": {}
            },
            "infrastructureSpec": {
                "apiVersion": "infrastructure.cluster.x-k8s.io/v1beta1",
                "kind": "RemoteMachine",
                "spec": {"address": "10.0.0.1", "port": 22}
            },
            "killIfCommands": []
        });
        let spec: ScheduledMachineSpec =
            serde_json::from_value(json).expect("empty killIfCommands must deserialize");
        assert_eq!(
            spec.kill_if_commands.as_deref(),
            Some([].as_slice()),
            "empty list must round-trip as Some(vec![]), not None"
        );
    }

    #[test]
    fn test_node_ref_round_trip_serialization() {
        let original = NodeRef {
            api_version: "v1".to_string(),
            kind: "Node".to_string(),
            name: "worker-03".to_string(),
            uid: Some("aaaa-bbbb".to_string()),
        };
        let json = serde_json::to_value(&original).expect("serialize NodeRef");
        assert_eq!(json["apiVersion"], "v1");
        assert_eq!(json["kind"], "Node");
        assert_eq!(json["name"], "worker-03");
        assert_eq!(json["uid"], "aaaa-bbbb");

        let round: NodeRef = serde_json::from_value(json).expect("round-trip NodeRef");
        assert_eq!(round.api_version, "v1");
        assert_eq!(round.uid.as_deref(), Some("aaaa-bbbb"));
    }

    // ========================================================================
    // NodeTaint / TaintEffect tests
    // ========================================================================

    #[test]
    fn test_node_taint_parse_with_value() {
        let json = serde_json::json!({
            "key": "workload",
            "value": "batch",
            "effect": "NoSchedule"
        });
        let taint: NodeTaint = serde_json::from_value(json).expect("parse NodeTaint with value");
        assert_eq!(taint.key, "workload");
        assert_eq!(taint.value.as_deref(), Some("batch"));
        assert_eq!(taint.effect, TaintEffect::NoSchedule);
    }

    #[test]
    fn test_node_taint_parse_without_value() {
        let json = serde_json::json!({
            "key": "dedicated",
            "effect": "NoExecute"
        });
        let taint: NodeTaint = serde_json::from_value(json).expect("parse NodeTaint no value");
        assert_eq!(taint.key, "dedicated");
        assert!(taint.value.is_none());
        assert_eq!(taint.effect, TaintEffect::NoExecute);
    }

    #[test]
    fn test_node_taint_round_trip_without_value_omits_field() {
        let taint = NodeTaint {
            key: "dedicated".to_string(),
            value: None,
            effect: TaintEffect::PreferNoSchedule,
        };
        let json = serde_json::to_value(&taint).expect("serialize");
        assert_eq!(json["key"], "dedicated");
        assert_eq!(json["effect"], "PreferNoSchedule");
        assert!(
            json.get("value").is_none(),
            "value=None must be omitted, got: {json}"
        );
    }

    #[test]
    fn test_taint_effect_rejects_invalid_variant() {
        let json = serde_json::json!({
            "key": "k",
            "effect": "Invalid"
        });
        let result: Result<NodeTaint, _> = serde_json::from_value(json);
        assert!(result.is_err(), "Invalid effect must fail to parse");
    }

    #[test]
    fn test_taint_effect_all_three_variants_round_trip() {
        for (variant, name) in [
            (TaintEffect::NoSchedule, "NoSchedule"),
            (TaintEffect::PreferNoSchedule, "PreferNoSchedule"),
            (TaintEffect::NoExecute, "NoExecute"),
        ] {
            let json = serde_json::to_value(&variant).expect("serialize");
            assert_eq!(json, serde_json::Value::String(name.to_string()));
            let round: TaintEffect = serde_json::from_value(json).expect("round-trip");
            assert_eq!(round, variant);
        }
    }

    #[test]
    fn test_node_taint_hash_eq_by_key_and_effect_and_value() {
        let a = NodeTaint {
            key: "k".to_string(),
            value: Some("v".to_string()),
            effect: TaintEffect::NoSchedule,
        };
        let b = a.clone();
        assert_eq!(a, b, "Clone must be Eq");
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b), "Hash must agree with Eq for clones");
    }

    #[test]
    fn test_scheduled_machine_spec_default_node_taints_is_empty() {
        let json = serde_json::json!({
            "schedule": {"daysOfWeek": [], "hoursOfDay": [], "timezone": "UTC", "enabled": true},
            "clusterName": "c",
            "bootstrapSpec": {
                "apiVersion": "bootstrap.cluster.x-k8s.io/v1beta1",
                "kind": "K0sWorkerConfig",
                "spec": {}
            },
            "infrastructureSpec": {
                "apiVersion": "infrastructure.cluster.x-k8s.io/v1beta1",
                "kind": "RemoteMachine",
                "spec": {"address": "10.0.0.1", "port": 22}
            }
        });
        let spec: ScheduledMachineSpec =
            serde_json::from_value(json).expect("missing nodeTaints must default to empty");
        assert!(spec.node_taints.is_empty());
    }

    #[test]
    fn test_scheduled_machine_spec_node_taints_omitted_when_empty() {
        let json = serde_json::json!({
            "schedule": {"daysOfWeek": [], "hoursOfDay": [], "timezone": "UTC", "enabled": true},
            "clusterName": "c",
            "bootstrapSpec": {
                "apiVersion": "bootstrap.cluster.x-k8s.io/v1beta1",
                "kind": "K0sWorkerConfig",
                "spec": {}
            },
            "infrastructureSpec": {
                "apiVersion": "infrastructure.cluster.x-k8s.io/v1beta1",
                "kind": "RemoteMachine",
                "spec": {"address": "10.0.0.1", "port": 22}
            }
        });
        let spec: ScheduledMachineSpec = serde_json::from_value(json).expect("parse");
        let back = serde_json::to_value(&spec).expect("serialize");
        assert!(
            back.get("nodeTaints").is_none(),
            "empty nodeTaints must serialize as omitted, got: {back}"
        );
    }

    #[test]
    fn test_scheduled_machine_spec_parses_node_taints() {
        let json = serde_json::json!({
            "schedule": {"daysOfWeek": [], "hoursOfDay": [], "timezone": "UTC", "enabled": true},
            "clusterName": "c",
            "bootstrapSpec": {
                "apiVersion": "bootstrap.cluster.x-k8s.io/v1beta1",
                "kind": "K0sWorkerConfig",
                "spec": {}
            },
            "infrastructureSpec": {
                "apiVersion": "infrastructure.cluster.x-k8s.io/v1beta1",
                "kind": "RemoteMachine",
                "spec": {"address": "10.0.0.1", "port": 22}
            },
            "nodeTaints": [
                {"key": "workload", "value": "batch", "effect": "NoSchedule"},
                {"key": "dedicated", "effect": "NoExecute"}
            ]
        });
        let spec: ScheduledMachineSpec = serde_json::from_value(json).expect("parse");
        assert_eq!(spec.node_taints.len(), 2);
        assert_eq!(spec.node_taints[0].key, "workload");
        assert_eq!(spec.node_taints[0].value.as_deref(), Some("batch"));
        assert_eq!(spec.node_taints[0].effect, TaintEffect::NoSchedule);
        assert_eq!(spec.node_taints[1].key, "dedicated");
        assert!(spec.node_taints[1].value.is_none());
        assert_eq!(spec.node_taints[1].effect, TaintEffect::NoExecute);
    }

    // --- validate_node_taints: happy path ---

    #[test]
    fn test_validate_node_taints_empty_list_ok() {
        assert!(validate_node_taints(&[]).is_ok());
    }

    #[test]
    fn test_validate_node_taints_simple_valid_list_ok() {
        let taints = vec![
            NodeTaint {
                key: "workload".to_string(),
                value: Some("batch".to_string()),
                effect: TaintEffect::NoSchedule,
            },
            NodeTaint {
                key: "dedicated".to_string(),
                value: None,
                effect: TaintEffect::NoExecute,
            },
        ];
        assert!(validate_node_taints(&taints).is_ok());
    }

    #[test]
    fn test_validate_node_taints_key_with_prefix_ok() {
        let taints = vec![NodeTaint {
            key: "example.com/team".to_string(),
            value: Some("platform".to_string()),
            effect: TaintEffect::NoSchedule,
        }];
        assert!(validate_node_taints(&taints).is_ok());
    }

    #[test]
    fn test_validate_node_taints_same_key_different_effect_ok() {
        // core/v1 allows same key with different effects.
        let taints = vec![
            NodeTaint {
                key: "workload".to_string(),
                value: Some("batch".to_string()),
                effect: TaintEffect::NoSchedule,
            },
            NodeTaint {
                key: "workload".to_string(),
                value: Some("batch".to_string()),
                effect: TaintEffect::NoExecute,
            },
        ];
        assert!(validate_node_taints(&taints).is_ok());
    }

    // --- validate_node_taints: rejection paths ---

    #[test]
    fn test_validate_node_taints_rejects_empty_key() {
        let taints = vec![NodeTaint {
            key: String::new(),
            value: None,
            effect: TaintEffect::NoSchedule,
        }];
        let err = validate_node_taints(&taints).expect_err("empty key must be rejected");
        assert!(err.contains("key"), "error should mention key: {err}");
    }

    #[test]
    fn test_validate_node_taints_rejects_leading_hyphen_key() {
        let taints = vec![NodeTaint {
            key: "-bad".to_string(),
            value: None,
            effect: TaintEffect::NoSchedule,
        }];
        let err = validate_node_taints(&taints).expect_err("leading hyphen key must be rejected");
        assert!(err.contains("key"), "error should mention key: {err}");
    }

    #[test]
    fn test_validate_node_taints_rejects_trailing_hyphen_key() {
        let taints = vec![NodeTaint {
            key: "bad-".to_string(),
            value: None,
            effect: TaintEffect::NoSchedule,
        }];
        let err = validate_node_taints(&taints).expect_err("trailing hyphen key must be rejected");
        assert!(err.contains("key"), "error should mention key: {err}");
    }

    #[test]
    fn test_validate_node_taints_rejects_key_with_invalid_char() {
        let taints = vec![NodeTaint {
            key: "bad$key".to_string(),
            value: None,
            effect: TaintEffect::NoSchedule,
        }];
        let err = validate_node_taints(&taints).expect_err("invalid char must be rejected");
        assert!(err.contains("key"), "error should mention key: {err}");
    }

    #[test]
    fn test_validate_node_taints_rejects_key_over_63_chars() {
        let long_key = "a".repeat(64);
        let taints = vec![NodeTaint {
            key: long_key,
            value: None,
            effect: TaintEffect::NoSchedule,
        }];
        let err = validate_node_taints(&taints).expect_err("long key must be rejected");
        assert!(err.contains("63"), "error should mention limit: {err}");
    }

    #[test]
    fn test_validate_node_taints_rejects_value_over_63_chars() {
        let long_value = "v".repeat(64);
        let taints = vec![NodeTaint {
            key: "workload".to_string(),
            value: Some(long_value),
            effect: TaintEffect::NoSchedule,
        }];
        let err = validate_node_taints(&taints).expect_err("long value must be rejected");
        assert!(err.contains("63"), "error should mention limit: {err}");
    }

    #[test]
    fn test_validate_node_taints_rejects_duplicate_key_and_effect() {
        let taints = vec![
            NodeTaint {
                key: "workload".to_string(),
                value: Some("a".to_string()),
                effect: TaintEffect::NoSchedule,
            },
            NodeTaint {
                key: "workload".to_string(),
                value: Some("b".to_string()),
                effect: TaintEffect::NoSchedule,
            },
        ];
        let err = validate_node_taints(&taints).expect_err("duplicate must be rejected");
        assert!(
            err.contains("duplicate"),
            "error should mention duplicate: {err}"
        );
    }

    #[test]
    fn test_validate_node_taints_rejects_reserved_5spot_prefix() {
        let taints = vec![NodeTaint {
            key: "5spot.finos.org/reserved".to_string(),
            value: None,
            effect: TaintEffect::NoSchedule,
        }];
        let err =
            validate_node_taints(&taints).expect_err("5spot.finos.org/ prefix must be rejected");
        assert!(
            err.contains("5spot.finos.org"),
            "error should mention reserved prefix: {err}"
        );
    }

    #[test]
    fn test_validate_node_taints_rejects_kubernetes_io_prefix() {
        let taints = vec![NodeTaint {
            key: "kubernetes.io/role".to_string(),
            value: None,
            effect: TaintEffect::NoSchedule,
        }];
        let err = validate_node_taints(&taints)
            .expect_err("kubernetes.io/ prefix must be rejected as reserved");
        assert!(
            err.contains("reserved"),
            "error should mention reserved: {err}"
        );
    }

    #[test]
    fn test_validate_node_taints_rejects_node_kubernetes_io_prefix() {
        let taints = vec![NodeTaint {
            key: "node.kubernetes.io/unreachable".to_string(),
            value: None,
            effect: TaintEffect::NoExecute,
        }];
        let err = validate_node_taints(&taints)
            .expect_err("node.kubernetes.io/ prefix must be rejected as reserved");
        assert!(
            err.contains("reserved"),
            "error should mention reserved: {err}"
        );
    }

    // ========================================================================
    // Phase 2 — status.appliedNodeTaints + NodeTainted condition constants
    // ========================================================================

    #[test]
    fn test_status_applied_node_taints_defaults_empty() {
        let status = ScheduledMachineStatus::default();
        assert!(status.applied_node_taints.is_empty());
    }

    #[test]
    fn test_status_applied_node_taints_omitted_when_empty() {
        let status = ScheduledMachineStatus::default();
        let json = serde_json::to_value(&status).expect("serialize");
        assert!(
            json.get("appliedNodeTaints").is_none(),
            "empty appliedNodeTaints must be omitted, got: {json}"
        );
    }

    #[test]
    fn test_status_applied_node_taints_round_trip() {
        let status = ScheduledMachineStatus {
            applied_node_taints: vec![
                NodeTaint {
                    key: "workload".to_string(),
                    value: Some("batch".to_string()),
                    effect: TaintEffect::NoSchedule,
                },
                NodeTaint {
                    key: "dedicated".to_string(),
                    value: None,
                    effect: TaintEffect::NoExecute,
                },
            ],
            ..Default::default()
        };
        let json = serde_json::to_value(&status).expect("serialize");
        assert_eq!(json["appliedNodeTaints"][0]["key"], "workload");
        assert_eq!(json["appliedNodeTaints"][0]["effect"], "NoSchedule");
        assert_eq!(json["appliedNodeTaints"][1]["key"], "dedicated");
        assert_eq!(json["appliedNodeTaints"][1]["effect"], "NoExecute");
        let round: ScheduledMachineStatus =
            serde_json::from_value(json).expect("deserialize status");
        assert_eq!(round.applied_node_taints.len(), 2);
        assert_eq!(round.applied_node_taints[0], status.applied_node_taints[0]);
        assert_eq!(round.applied_node_taints[1], status.applied_node_taints[1]);
    }

    #[test]
    fn test_status_missing_applied_node_taints_deserializes_as_empty() {
        let json = serde_json::json!({});
        let status: ScheduledMachineStatus =
            serde_json::from_value(json).expect("deserialize defaulted status");
        assert!(status.applied_node_taints.is_empty());
    }

    #[test]
    fn test_node_tainted_condition_constants() {
        use crate::constants::{
            CONDITION_TYPE_NODE_TAINTED, REASON_NODE_NOT_READY, REASON_NODE_TAINTS_APPLIED,
            REASON_NODE_TAINT_PATCH_FAILED, REASON_NO_NODE_YET, REASON_TAINT_OWNERSHIP_CONFLICT,
        };
        assert_eq!(CONDITION_TYPE_NODE_TAINTED, "NodeTainted");
        assert_eq!(REASON_NODE_TAINTS_APPLIED, "Applied");
        assert_eq!(REASON_NODE_NOT_READY, "NodeNotReady");
        assert_eq!(REASON_NODE_TAINT_PATCH_FAILED, "PatchFailed");
        assert_eq!(REASON_NO_NODE_YET, "NoNodeYet");
        assert_eq!(REASON_TAINT_OWNERSHIP_CONFLICT, "TaintOwnershipConflict");
    }

    #[test]
    fn test_validate_node_taints_rejects_node_role_kubernetes_io_prefix() {
        let taints = vec![NodeTaint {
            key: "node-role.kubernetes.io/control-plane".to_string(),
            value: None,
            effect: TaintEffect::NoSchedule,
        }];
        let err = validate_node_taints(&taints)
            .expect_err("node-role.kubernetes.io/ prefix must be rejected as reserved");
        assert!(
            err.contains("reserved") || err.contains("machineTemplate"),
            "error should explain: {err}"
        );
    }
}
