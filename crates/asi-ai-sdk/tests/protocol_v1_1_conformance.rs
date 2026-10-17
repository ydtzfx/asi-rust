use asi_ai_sdk::protocol_v1_1 as protocol;
use serde_json::Value;
use std::{env, fs};

fn vector(case_id: &str) -> Value {
    let path = env::var("AGENT_PROTOCOL_VECTORS")
        .expect("AGENT_PROTOCOL_VECTORS must point to agi-platform canonical vectors");
    let document: Value = serde_json::from_str(&fs::read_to_string(path).expect("read vector file"))
        .expect("parse vector file");
    assert_eq!(document["protocol_version"], "1.1");
    document["cases"]
        .as_array()
        .expect("cases array")
        .iter()
        .find(|case| case["id"] == case_id)
        .cloned()
        .unwrap_or_else(|| panic!("missing vector {case_id}"))
}

fn run(case_id: &str) {
    let case = vector(case_id);
    let input = &case["input"];
    let expected = &case["expect"];

    match case_id {
        "C001" => {
            let result = protocol::authorize_workspace_path(
                input["workspace"].as_str().unwrap(),
                input["requested_path"].as_str().unwrap(),
            );
            assert_eq!(result["allowed"], expected["allowed"]);
            assert_eq!(result["code"], expected["code"]);
        }
        "C002" => {
            let result = protocol::authorize_resolved_path(
                input["workspace"].as_str().unwrap(),
                input["requested_path"].as_str().unwrap(),
                input["resolved_path"].as_str().unwrap(),
            );
            assert_eq!(result["allowed"], expected["allowed"]);
            assert_eq!(result["code"], expected["code"]);
        }
        "C003" => {
            let result = protocol::authorize_non_sensitive_path(
                input["workspace"].as_str().unwrap(),
                input["requested_path"].as_str().unwrap(),
            );
            assert_eq!(result["allowed"], expected["allowed"]);
            assert_eq!(result["code"], expected["code"]);
            assert_eq!(result["segment"], expected["segment"]);
        }
        "C004" => {
            let required: Vec<String> = serde_json::from_value(input["required"].clone()).unwrap();
            let granted: Vec<String> = serde_json::from_value(input["granted"].clone()).unwrap();
            let result = protocol::authorize_capabilities(&required, &granted);
            assert_eq!(result["allowed"], expected["allowed"]);
            assert_eq!(result["missing"], expected["missing"]);
        }
        "C005" => {
            let result = protocol::apply_approval_gate(
                &input["run"],
                &input["approval"],
                input["now"].as_str().unwrap(),
            );
            assert_eq!(result["state"], expected["state"]);
            assert_eq!(result["sequence"], expected["sequence"]);
            assert_eq!(result["reason_code"], expected["reason_code"]);
        }
        "C006" => {
            let result = protocol::resume_approved_run(&input["run"], &input["approval"]).unwrap();
            assert_eq!(result["run_id"], expected["run_id"]);
            assert_eq!(result["state"], expected["state"]);
            assert_eq!(result["sequence"], expected["sequence"]);
            assert_eq!(result["reason_code"], expected["reason_code"]);
        }
        "C007" => {
            let result = protocol::apply_tool_timeout(
                &input["run"],
                input["tool_call_id"].as_str().unwrap(),
                input["now"].as_str().unwrap(),
            );
            assert_eq!(result["state"], expected["state"]);
            assert_eq!(result["sequence"], expected["sequence"]);
            assert_eq!(result["reason_code"], expected["reason_code"]);
        }
        "C008" => {
            let result = protocol::execute_cooperative_steps(
                input["max_steps"].as_u64().unwrap() as usize,
                input["cancel_after_step"].as_u64().unwrap() as usize,
            );
            assert_eq!(result, *expected);
        }
        "C009" => {
            let event = protocol::build_redacted_audit_event(input);
            assert!(!serde_json::to_string(&event)
                .unwrap()
                .contains(expected["secret_absent"].as_str().unwrap()));
            assert_eq!(event["redacted_fields"], expected["redacted_fields"]);
            assert_eq!(event["input_digest"], expected["input_digest"]);
        }
        "C010" => {
            let envelope = protocol::wrap_tool_output(
                input["tool_call_id"].as_str().unwrap(),
                &input["output"],
            );
            assert_eq!(envelope["source"], expected["source"]);
            assert_eq!(envelope["trust"], expected["trust"]);
            assert_eq!(envelope["interpretation"], expected["interpretation"]);
            assert_eq!(
                protocol::require_trusted_control_input(&envelope).unwrap_err(),
                expected["control_code"].as_str().unwrap()
            );
        }
        "C011" => {
            let mut budget = protocol::StepBudget::new(input["limit"].as_u64().unwrap() as usize)
                .unwrap();
            let mut result = Value::Null;
            for _ in 0..input["attempts"].as_u64().unwrap() {
                result = budget.consume();
            }
            assert_eq!(result, *expected);
        }
        "C012" => {
            let result = protocol::verify_mutation_precondition(
                input["expected"].as_str().unwrap(),
                input["current"].as_str().unwrap(),
            );
            assert_eq!(result, *expected);
        }
        "C013" => {
            let mut journal = protocol::IdempotencyJournal::default();
            let mut effects = 0;
            let key = input["idempotency_key"].as_str().unwrap();
            let effect_result = input["effect_result"].clone();
            let first = journal
                .execute(key, || {
                    effects += 1;
                    effect_result.clone()
                })
                .unwrap();
            let second = journal
                .execute(key, || {
                    effects += 1;
                    effect_result.clone()
                })
                .unwrap();
            assert_eq!(first["replayed"], expected["first_replayed"]);
            assert_eq!(second["replayed"], expected["second_replayed"]);
            assert_eq!(effects, expected["effects"].as_u64().unwrap());
        }
        "C014" => {
            let result = protocol::enter_recovery(
                &input["operation"],
                input["now"].as_str().unwrap(),
            )
            .unwrap();
            assert_eq!(result["status"], expected["status"]);
            assert_eq!(result["attempt"], expected["attempt"]);
            assert_eq!(result["recovery"]["strategy"], expected["strategy"]);
            assert_eq!(result["error"]["retryable"], expected["retryable"]);
        }
        "C015" => {
            let events = input["events"].as_array().unwrap();
            let prefix = protocol::validate_audit_correlation(&events[..2]);
            let mismatch = protocol::validate_audit_correlation(events);
            assert_eq!(prefix["valid"], expected["valid_prefix"]);
            assert_eq!(mismatch["valid"], expected["mismatch_valid"]);
            assert_eq!(mismatch["code"], expected["mismatch_code"]);
        }
        "C016" => {
            let declared: Vec<String> = serde_json::from_value(input["declared"].clone()).unwrap();
            let allowed = protocol::authorize_tool(
                input["allowed_request"].as_str().unwrap(),
                &declared,
            );
            let denied = protocol::authorize_tool(
                input["denied_request"].as_str().unwrap(),
                &declared,
            );
            assert_eq!(allowed["allowed"], expected["allowed"]);
            assert_eq!(denied["allowed"], expected["denied"]);
            assert_eq!(denied["code"], expected["code"]);
        }
        _ => panic!("unknown vector {case_id}"),
    }
    println!("PASS {case_id} {}", case["name"].as_str().unwrap());
}

macro_rules! vector_test {
    ($name:ident, $id:literal) => {
        #[test]
        #[ignore = "requires agi-platform canonical vectors"]
        fn $name() {
            run($id);
        }
    };
}

vector_test!(c001, "C001");
vector_test!(c002, "C002");
vector_test!(c003, "C003");
vector_test!(c004, "C004");
vector_test!(c005, "C005");
vector_test!(c006, "C006");
vector_test!(c007, "C007");
vector_test!(c008, "C008");
vector_test!(c009, "C009");
vector_test!(c010, "C010");
vector_test!(c011, "C011");
vector_test!(c012, "C012");
vector_test!(c013, "C013");
vector_test!(c014, "C014");
vector_test!(c015, "C015");
vector_test!(c016, "C016");
