use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn provenance_families_use_shared_command_input_helpers() {
    for path in [
        "src/cmd_pretraining_source_provenance_receipt.rs",
        "src/cmd_pretraining_source_provenance_evidence_bundle.rs",
        "src/cmd_profiling_provenance_receipt.rs",
        "src/cmd_profiling_provenance_bundle.rs",
    ] {
        let text = repo_file(path);
        for forbidden in [
            "fn load_json_object(",
            "fn read_string(",
            "fn read_u64(",
            "fn read_array(",
            "fn read_object(",
            "fn read_string_array(",
            "fn load_history(",
            "fn history_entries_mut(",
            "fn write_json(",
        ] {
            assert!(
                !text.contains(forbidden),
                "{path} must use shared command-input helpers instead of local `{forbidden}`"
            );
        }
    }
}
