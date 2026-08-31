use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn bitctx() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bitctx"))
}

fn run(data_dir: &Path, args: &[&str]) -> Output {
    bitctx()
        .arg("--data-dir")
        .arg(data_dir)
        .args(args)
        .output()
        .expect("bitctx should run")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_schema(directory: &Path) -> PathBuf {
    let path = directory.join("schema.json");
    fs::write(
        &path,
        r#"{
  "version": 1,
  "bits": {
    "0": {"name": "auth", "desc": "User is authenticated"},
    "1": {"name": "permission", "desc": "Permission is verified"},
    "3": {"name": "승인", "desc": "한국어 설명"}
  },
  "masks": {
    "required": {"bits": [3, 0, 1], "desc": "Required conditions"}
  }
}
"#,
    )
    .expect("schema should be written");
    path
}

fn write_multi_mask_schema(directory: &Path, default_mask: Option<&str>) -> PathBuf {
    let path = directory.join("multi-schema.json");
    let default_mask = default_mask
        .map(|name| format!("  \"default_mask\": \"{name}\",\n"))
        .unwrap_or_default();
    fs::write(
        &path,
        format!(
            r#"{{
  "version": 1,
{default_mask}  "bits": {{
    "0": {{"name": "auth", "desc": "User is authenticated"}},
    "1": {{"name": "permission", "desc": "Permission is verified"}}
  }},
  "masks": {{
    "complete": {{"bits": [0, 1], "desc": "Complete conditions"}},
    "required": {{"bits": [1], "desc": "Required conditions"}}
  }}
}}
"#
        ),
    )
    .expect("multi-mask schema should be written");
    path
}

fn initialize(data_dir: &Path, schema_path: &Path, session: &str) {
    let output = run(
        data_dir,
        &[
            "init",
            "--session",
            session,
            "--schema",
            schema_path.to_str().expect("UTF-8 path"),
        ],
    );
    assert_success(&output);
}

#[test]
fn full_flow_starts_false_and_ends_true() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = write_schema(temp.path());
    initialize(&data_dir, &schema, "flow");

    let initial = run(
        &data_dir,
        &[
            "eval",
            "--session",
            "flow",
            "--mask",
            "required",
            "--format",
            "json",
        ],
    );
    assert_success(&initial);
    let initial_json: Value = serde_json::from_slice(&initial.stdout).expect("valid JSON output");
    assert_eq!(initial_json["pass"], false);
    assert_eq!(initial_json["missing"], serde_json::json!([3, 0, 1]));
    assert_eq!(
        initial_json["missing_labels"],
        serde_json::json!(["승인", "auth", "permission"])
    );
    assert_eq!(initial_json["missing_conditions"][0]["index"], 3);
    assert_eq!(initial_json["missing_conditions"][0]["name"], "승인");
    assert_eq!(initial_json["missing_conditions"][0]["desc"], "한국어 설명");

    let set = run(
        &data_dir,
        &[
            "set",
            "--session",
            "flow",
            "--bit",
            "승인,auth,permission",
            "--value",
            "true,true,true",
        ],
    );
    assert_success(&set);

    let final_eval = run(
        &data_dir,
        &["eval", "--session", "flow", "--mask", "required"],
    );
    assert_success(&final_eval);
    let final_json: Value = serde_json::from_slice(&final_eval.stdout).expect("valid JSON output");
    assert_eq!(final_json["pass"], true);
    assert_eq!(final_json["missing"], serde_json::json!([]));
    assert_eq!(final_json["missing_conditions"], serde_json::json!([]));

    let explain = run(
        &data_dir,
        &[
            "explain",
            "--session",
            "flow",
            "--mask",
            "required",
            "--lang",
            "en",
        ],
    );
    assert_success(&explain);
    assert!(String::from_utf8_lossy(&explain.stdout).contains("All conditions satisfied"));

    let dump = run(
        &data_dir,
        &["dump", "--session", "flow", "--format", "json"],
    );
    assert_success(&dump);
    let dump_json: Value = serde_json::from_slice(&dump.stdout).expect("valid JSON output");
    assert_eq!(dump_json["bits"], 11);
    assert_eq!(dump_json["bit_states"][0]["index"], 0);
    assert_eq!(dump_json["bit_states"][1]["index"], 1);
    assert_eq!(dump_json["bit_states"][2]["index"], 3);

    let reset = run(&data_dir, &["reset", "--session", "flow", "--force"]);
    assert_success(&reset);
    assert!(!data_dir.join("flow").exists());
    assert!(data_dir.join(".locks/flow.lock").exists());
}

#[test]
fn resume_restores_missing_state_without_replaying_settled_bits() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = write_schema(temp.path());
    initialize(&data_dir, &schema, "resume-flow");
    assert_success(&run(
        &data_dir,
        &[
            "set",
            "--session",
            "resume-flow",
            "--bit",
            "auth",
            "--value",
            "true",
        ],
    ));

    let output = run(&data_dir, &["resume", "--session", "resume-flow"]);
    assert_success(&output);
    let result: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(result["session_id"], "resume-flow");
    assert_eq!(result["mask"], "required");
    assert_eq!(result["pass"], false);
    assert_eq!(result["missing"], serde_json::json!([3, 1]));
    assert_eq!(
        result["missing_labels"],
        serde_json::json!(["승인", "permission"])
    );
    assert_eq!(result["freshness"], "unverified");
    assert!(result.get("bit_states").is_none());

    let text = run(
        &data_dir,
        &["resume", "--session", "resume-flow", "--format", "text"],
    );
    assert_success(&text);
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("Session: resume-flow\n"));
    assert!(stdout.contains("Mask: required\n"));
    assert!(stdout.contains("Freshness: unverified\n"));
    assert!(stdout.contains("RESULT: X\n"));
    assert!(stdout.contains("X bit 3: 승인 (한국어 설명)\n"));
    assert!(!stdout.contains("auth (User is authenticated)"));
}

#[test]
fn resume_uses_default_mask_and_allows_explicit_override() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = write_multi_mask_schema(temp.path(), Some("required"));
    initialize(&data_dir, &schema, "resume-default");

    let default = run(&data_dir, &["resume", "--session", "resume-default"]);
    assert_success(&default);
    let result: Value = serde_json::from_slice(&default.stdout).expect("valid JSON output");
    assert_eq!(result["mask"], "required");
    assert_eq!(result["missing"], serde_json::json!([1]));

    let explicit = run(
        &data_dir,
        &[
            "resume",
            "--session",
            "resume-default",
            "--mask",
            "complete",
        ],
    );
    assert_success(&explicit);
    let result: Value = serde_json::from_slice(&explicit.stdout).expect("valid JSON output");
    assert_eq!(result["mask"], "complete");
    assert_eq!(result["missing"], serde_json::json!([0, 1]));
}

#[test]
fn resume_rejects_ambiguous_or_invalid_default_masks() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let ambiguous_schema = write_multi_mask_schema(temp.path(), None);
    initialize(&data_dir, &ambiguous_schema, "resume-ambiguous");

    let ambiguous = run(&data_dir, &["resume", "--session", "resume-ambiguous"]);
    assert_failure(&ambiguous);
    let stderr = String::from_utf8_lossy(&ambiguous.stderr);
    assert!(stderr.contains("requires --mask"));
    assert!(stderr.contains("complete, required"));

    let unknown = run(
        &data_dir,
        &[
            "resume",
            "--session",
            "resume-ambiguous",
            "--mask",
            "unknown",
        ],
    );
    assert_failure(&unknown);
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("mask 'unknown' not found"));

    let invalid_format = run(
        &data_dir,
        &[
            "resume",
            "--session",
            "resume-ambiguous",
            "--mask",
            "required",
            "--format",
            "yaml",
        ],
    );
    assert_failure(&invalid_format);
    assert!(
        String::from_utf8_lossy(&invalid_format.stderr)
            .contains("unknown format 'yaml': use json or text")
    );

    let invalid_schema = write_multi_mask_schema(temp.path(), Some("unknown"));
    let invalid = run(
        &data_dir,
        &[
            "init",
            "--session",
            "resume-invalid",
            "--schema",
            invalid_schema.to_str().expect("UTF-8 path"),
        ],
    );
    assert_failure(&invalid);
    assert!(
        String::from_utf8_lossy(&invalid.stderr)
            .contains("default mask 'unknown' not found in schema")
    );
}

#[test]
fn text_eval_renders_fixed_matrix_and_filtered_details() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = write_schema(temp.path());
    initialize(&data_dir, &schema, "matrix");
    assert_success(&run(
        &data_dir,
        &[
            "set",
            "--session",
            "matrix",
            "--bit",
            "auth",
            "--value",
            "true",
        ],
    ));

    let output = run(
        &data_dir,
        &[
            "eval",
            "--session",
            "matrix",
            "--mask",
            "required",
            "--format",
            "text",
        ],
    );
    assert_success(&output);
    let expected = concat!(
        "     0   1   2   3   4   5   6   7\n",
        "00 ┌───┬───┬───┬───┬───┬───┬───┬───┐\n",
        "   │ O │ X │ · │ X │ · │ · │ · │ · │\n",
        "08 ├───┼───┼───┼───┼───┼───┼───┼───┤\n",
        "   │ · │ · │ · │ · │ · │ · │ · │ · │\n",
        "16 ├───┼───┼───┼───┼───┼───┼───┼───┤\n",
        "   │ · │ · │ · │ · │ · │ · │ · │ · │\n",
        "24 ├───┼───┼───┼───┼───┼───┼───┼───┤\n",
        "   │ · │ · │ · │ · │ · │ · │ · │ · │\n",
        "32 ├───┼───┼───┼───┼───┼───┼───┼───┤\n",
        "   │ · │ · │ · │ · │ · │ · │ · │ · │\n",
        "40 ├───┼───┼───┼───┼───┼───┼───┼───┤\n",
        "   │ · │ · │ · │ · │ · │ · │ · │ · │\n",
        "48 ├───┼───┼───┼───┼───┼───┼───┼───┤\n",
        "   │ · │ · │ · │ · │ · │ · │ · │ · │\n",
        "56 ├───┼───┼───┼───┼───┼───┼───┼───┤\n",
        "   │ · │ · │ · │ · │ · │ · │ · │ · │\n",
        "   └───┴───┴───┴───┴───┴───┴───┴───┘\n",
        "\n",
        "RESULT: X\n",
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);

    let cases = [
        (
            "all",
            concat!(
                "\nDETAILS (all)\n",
                "  X bit 3: 승인 (한국어 설명)\n",
                "  O bit 0: auth (User is authenticated)\n",
                "  X bit 1: permission (Permission is verified)\n",
            ),
        ),
        (
            "satisfied",
            concat!(
                "\nDETAILS (satisfied)\n",
                "  O bit 0: auth (User is authenticated)\n",
            ),
        ),
        (
            "missing",
            concat!(
                "\nDETAILS (missing)\n",
                "  X bit 3: 승인 (한국어 설명)\n",
                "  X bit 1: permission (Permission is verified)\n",
            ),
        ),
    ];

    for (filter, details) in cases {
        let output = run(
            &data_dir,
            &[
                "eval",
                "--session",
                "matrix",
                "--mask",
                "required",
                "--format",
                "text",
                "--show",
                filter,
            ],
        );
        assert_success(&output);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            format!("{expected}{details}")
        );
    }
}

#[test]
fn text_eval_reports_success_and_empty_filtered_details() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = write_schema(temp.path());
    initialize(&data_dir, &schema, "matrix-pass");
    assert_success(&run(
        &data_dir,
        &[
            "set",
            "--session",
            "matrix-pass",
            "--bit",
            "승인,auth,permission",
            "--value",
            "true,true,true",
        ],
    ));

    let output = run(
        &data_dir,
        &[
            "eval",
            "--session",
            "matrix-pass",
            "--mask",
            "required",
            "--format",
            "text",
            "--show",
            "missing",
        ],
    );
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RESULT: O\n"));
    assert!(stdout.ends_with("\nDETAILS (missing)\n  (none)\n"));
}

#[test]
fn show_filter_requires_text_format() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = write_schema(temp.path());
    initialize(&data_dir, &schema, "invalid-show");

    let output = run(
        &data_dir,
        &[
            "eval",
            "--session",
            "invalid-show",
            "--mask",
            "required",
            "--show",
            "all",
        ],
    );
    assert_failure(&output);
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("--show can only be used with --format text")
    );
}

#[test]
fn rejects_path_escape_ids_without_touching_outside_state() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = write_schema(temp.path());
    let sentinel = temp.path().join("sentinel");
    fs::create_dir(&sentinel).expect("sentinel should be created");
    fs::write(sentinel.join("keep"), "safe").expect("sentinel should be written");

    let absolute = temp.path().join("outside");
    let absolute_id = absolute.to_str().expect("UTF-8 path").to_string();
    let overlong = "a".repeat(129);
    let invalid_ids = [
        ".".to_string(),
        "..".to_string(),
        "../sentinel".to_string(),
        "a/b".to_string(),
        r"a\b".to_string(),
        "a\nb".to_string(),
        absolute_id,
        overlong,
    ];

    for session_id in invalid_ids {
        let output = run(&data_dir, &["reset", "--session", &session_id, "--force"]);
        assert_failure(&output);

        let output = run(
            &data_dir,
            &[
                "init",
                "--session",
                &session_id,
                "--schema",
                schema.to_str().expect("UTF-8 path"),
                "--force",
            ],
        );
        assert_failure(&output);
    }

    assert_eq!(
        fs::read_to_string(sentinel.join("keep")).expect("sentinel should remain"),
        "safe"
    );
    assert!(!absolute.exists());
    assert!(!data_dir.exists());
}

#[test]
fn force_init_resets_bits_and_set_requires_initialization() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = write_schema(temp.path());

    let missing_set = run(
        &data_dir,
        &[
            "set",
            "--session",
            "missing",
            "--bit",
            "auth",
            "--value",
            "true",
        ],
    );
    assert_failure(&missing_set);
    assert!(!data_dir.join("missing").exists());

    initialize(&data_dir, &schema, "force");
    assert_success(&run(
        &data_dir,
        &[
            "set",
            "--session",
            "force",
            "--bit",
            "auth",
            "--value",
            "true",
        ],
    ));
    assert_success(&run(
        &data_dir,
        &[
            "init",
            "--session",
            "force",
            "--schema",
            schema.to_str().expect("UTF-8 path"),
            "--force",
        ],
    ));

    let dump = run(
        &data_dir,
        &["dump", "--session", "force", "--format", "json"],
    );
    assert_success(&dump);
    let state: Value = serde_json::from_slice(&dump.stdout).expect("valid JSON output");
    assert_eq!(state["bits"], 0);
}

#[test]
fn schema_hash_mismatch_is_explicit() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = write_schema(temp.path());
    initialize(&data_dir, &schema, "mismatch");

    let stored_schema = data_dir.join("mismatch/schema.json");
    let contents = fs::read_to_string(&stored_schema).expect("stored schema should exist");
    fs::write(
        &stored_schema,
        contents.replace("User is authenticated", "Changed description"),
    )
    .expect("stored schema should be changed");

    let output = run(
        &data_dir,
        &["eval", "--session", "mismatch", "--mask", "required"],
    );
    assert_failure(&output);
    assert!(String::from_utf8_lossy(&output.stderr).contains("schema hash mismatch"));
}

#[cfg(unix)]
#[test]
fn reset_and_force_init_refuse_symlink_session_paths() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let outside = temp.path().join("outside");
    let schema = write_schema(temp.path());
    fs::create_dir_all(&data_dir).expect("data directory should be created");
    fs::create_dir_all(&outside).expect("outside directory should be created");
    fs::write(outside.join("keep"), "safe").expect("sentinel should be written");
    symlink(&outside, data_dir.join("linked")).expect("session symlink should be created");

    let reset = run(&data_dir, &["reset", "--session", "linked", "--force"]);
    assert_failure(&reset);
    let init = run(
        &data_dir,
        &[
            "init",
            "--session",
            "linked",
            "--schema",
            schema.to_str().expect("UTF-8 path"),
            "--force",
        ],
    );
    assert_failure(&init);

    assert_eq!(
        fs::read_to_string(outside.join("keep")).expect("sentinel should remain"),
        "safe"
    );
    assert!(data_dir.join("linked").is_symlink());
}

#[test]
fn concurrent_set_operations_do_not_lose_updates() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = write_schema(temp.path());
    initialize(&data_dir, &schema, "concurrent");

    let mut first = bitctx();
    first
        .arg("--data-dir")
        .arg(&data_dir)
        .args([
            "set",
            "--session",
            "concurrent",
            "--bit",
            "auth",
            "--value",
            "true",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut second = bitctx();
    second
        .arg("--data-dir")
        .arg(&data_dir)
        .args([
            "set",
            "--session",
            "concurrent",
            "--bit",
            "permission",
            "--value",
            "true",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let first_child = first.spawn().expect("first set should spawn");
    let second_child = second.spawn().expect("second set should spawn");
    assert_success(
        &first_child
            .wait_with_output()
            .expect("first set should finish"),
    );
    assert_success(
        &second_child
            .wait_with_output()
            .expect("second set should finish"),
    );

    let dump = run(
        &data_dir,
        &["dump", "--session", "concurrent", "--format", "json"],
    );
    assert_success(&dump);
    let state: Value = serde_json::from_slice(&dump.stdout).expect("valid JSON output");
    assert_eq!(state["bits"], 3);
}

#[test]
fn cli_data_dir_overrides_environment() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let cli_data = temp.path().join("cli-data");
    let env_data = temp.path().join("env-data");
    let schema = write_schema(temp.path());

    let output = bitctx()
        .env("BITCTX_DATA_DIR", &env_data)
        .arg("--data-dir")
        .arg(&cli_data)
        .args([
            "init",
            "--session",
            "precedence",
            "--schema",
            schema.to_str().expect("UTF-8 path"),
        ])
        .output()
        .expect("bitctx should run");
    assert_success(&output);
    assert!(cli_data.join("precedence/session.json").exists());
    assert!(!env_data.exists());
}

#[test]
fn duplicate_json_keys_are_rejected_by_init() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let schema = temp.path().join("duplicate.json");
    fs::write(
        &schema,
        r#"{
          "version": 1,
          "bits": {
            "1": {"name": "first", "desc": ""},
            "01": {"name": "second", "desc": ""}
          },
          "masks": {"m": {"bits": [1], "desc": ""}}
        }"#,
    )
    .expect("schema should be written");

    let output = run(
        &data_dir,
        &[
            "init",
            "--session",
            "duplicate",
            "--schema",
            schema.to_str().expect("UTF-8 path"),
        ],
    );
    assert_failure(&output);
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate bit index"));
    assert!(!data_dir.join("duplicate").exists());
}

#[test]
fn v01_session_format_remains_readable() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let data_dir = temp.path().join("data");
    let session_dir = data_dir.join("compatibility");
    fs::create_dir_all(&session_dir).expect("v0.1 session directory should be created");
    fs::write(
        session_dir.join("schema.json"),
        r#"{
  "version": 1,
  "bits": {
    "0": {"name": "user_authenticated", "desc": "사용자 인증 완료"},
    "1": {"name": "has_permission", "desc": "필요 권한 보유"},
    "2": {"name": "resource_exists", "desc": "대상 리소스 존재"},
    "3": {"name": "quota_ok", "desc": "쿼터 초과 안 함"},
    "4": {"name": "rate_limit_ok", "desc": "레이트리밋 여유"}
  },
  "masks": {
    "required": {"bits": [0, 1, 3], "desc": "기본 실행 필수 조건"},
    "admin_only": {"bits": [0, 1], "desc": "관리자 전용"},
    "read_access": {"bits": [0, 2], "desc": "읽기 권한"}
  }
}
"#,
    )
    .expect("v0.1 schema should be written");
    fs::write(
        session_dir.join("session.json"),
        r#"{
  "id": "compatibility",
  "schema_hash": "28a6fcebf6e2a82b",
  "bits": 1,
  "created_at": "2026-08-29T11:00:11.685587+00:00",
  "updated_at": "2026-08-29T11:00:11.685591+00:00"
}
"#,
    )
    .expect("v0.1 session state should be written");

    let output = run(
        &data_dir,
        &["dump", "--session", "compatibility", "--format", "json"],
    );
    assert_success(&output);
    let state: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(state["schema_hash"], "28a6fcebf6e2a82b");
    assert_eq!(state["bits"], 1);
    assert_eq!(state["bit_states"][0]["name"], "user_authenticated");
    assert_eq!(state["bit_states"][0]["value"], true);
}
