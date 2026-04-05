use std::fs;

use serde_json::Value;

#[test]
fn deploy_promote_current_and_rollback_current_pointer_update_files_and_records() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let current_pointer = tmp.path().join("artifacts/inference/current");
    let previous_version_file = tmp
        .path()
        .join("artifacts/deploy/previous_current_version.txt");
    let promotion_record = tmp
        .path()
        .join("artifacts/deploy/inference_current_pointer_promotion.json");
    let rollback_record = tmp
        .path()
        .join("artifacts/deploy/inference_current_pointer_rollback.json");

    fs::create_dir_all(
        current_pointer
            .parent()
            .expect("current pointer parent directory"),
    )
    .expect("create current pointer parent");
    fs::write(&current_pointer, "0.1.0\n").expect("write current pointer");

    let mut promote = assert_cmd::cargo::cargo_bin_cmd!("afterburner");
    promote
        .current_dir(tmp.path())
        .arg("deploy")
        .arg("promote-current")
        .arg("--artifact-version")
        .arg("0.2.0")
        .arg("--current-pointer")
        .arg(&current_pointer)
        .arg("--previous-version-file")
        .arg(&previous_version_file)
        .arg("--out-record")
        .arg(&promotion_record)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(&current_pointer).expect("read promoted pointer"),
        "0.2.0\n"
    );
    assert_eq!(
        fs::read_to_string(&previous_version_file).expect("read saved previous version"),
        "0.1.0\n"
    );

    let promotion: Value = serde_json::from_str(
        &fs::read_to_string(&promotion_record).expect("read promotion record"),
    )
    .expect("parse promotion record");
    assert_eq!(promotion["previous_version"], "0.1.0");
    assert_eq!(promotion["promoted_version"], "0.2.0");
    assert_eq!(
        promotion["current_pointer_path"],
        Value::from(current_pointer.display().to_string())
    );

    let mut rollback = assert_cmd::cargo::cargo_bin_cmd!("afterburner");
    rollback
        .current_dir(tmp.path())
        .arg("rollback")
        .arg("current-pointer")
        .arg("--current-pointer")
        .arg(&current_pointer)
        .arg("--previous-version-file")
        .arg(&previous_version_file)
        .arg("--out-record")
        .arg(&rollback_record)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(&current_pointer).expect("read rolled back pointer"),
        "0.1.0\n"
    );

    let rollback: Value =
        serde_json::from_str(&fs::read_to_string(&rollback_record).expect("read rollback record"))
            .expect("parse rollback record");
    assert_eq!(rollback["previous_version"], "0.2.0");
    assert_eq!(rollback["restored_version"], "0.1.0");
    assert_eq!(
        rollback["previous_version_path"],
        Value::from(previous_version_file.display().to_string())
    );
}
