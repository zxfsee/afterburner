use std::path::Path;

use crate::command_artifacts::{JsonArtifactError, write_json_value};
use serde_json::{Map, Value};
use std::fs;

#[derive(Debug)]
pub enum CommandInputJsonError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for CommandInputJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CommandInputJsonError {}

impl From<std::io::Error> for CommandInputJsonError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for CommandInputJsonError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!("serialize json: {err}")),
        }
    }
}

pub fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<Map<String, Value>, CommandInputJsonError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        CommandInputJsonError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        CommandInputJsonError::Parse(format!("{kind} `{}` must be an object", path.display()))
    })
}

pub fn read_string(
    object: &Map<String, Value>,
    key: &str,
    path: &Path,
    kind: &str,
) -> Result<String, CommandInputJsonError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            CommandInputJsonError::Parse(format!(
                "{kind} `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

pub fn read_non_empty_string(
    object: &Map<String, Value>,
    key: &str,
    path: &Path,
    kind: &str,
) -> Result<String, CommandInputJsonError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            CommandInputJsonError::Parse(format!(
                "{kind} `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

pub fn read_u64(
    object: &Map<String, Value>,
    key: &str,
    path: &Path,
    kind: &str,
) -> Result<u64, CommandInputJsonError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        CommandInputJsonError::Parse(format!(
            "{kind} `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

pub fn read_array(
    object: &Map<String, Value>,
    key: &str,
    path: &Path,
    kind: &str,
) -> Result<Value, CommandInputJsonError> {
    object
        .get(key)
        .filter(|value| value.is_array())
        .cloned()
        .ok_or_else(|| {
            CommandInputJsonError::Parse(format!(
                "{kind} `{}` missing array field `{key}`",
                path.display()
            ))
        })
}

pub fn read_object(
    object: &Map<String, Value>,
    key: &str,
    path: &Path,
    kind: &str,
) -> Result<Value, CommandInputJsonError> {
    object.get(key).cloned().ok_or_else(|| {
        CommandInputJsonError::Parse(format!(
            "{kind} `{}` missing field `{key}`",
            path.display()
        ))
    })
}

pub fn read_string_array(
    object: &Map<String, Value>,
    key: &str,
    path: &Path,
    kind: &str,
) -> Result<Vec<String>, CommandInputJsonError> {
    object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CommandInputJsonError::Parse(format!(
                "{kind} `{}` missing array field `{key}`",
                path.display()
            ))
        })?
        .iter()
        .map(|value| {
            value.as_str().map(str::to_string).ok_or_else(|| {
                CommandInputJsonError::Parse(format!(
                    "{kind} `{}` field `{key}` must contain only strings",
                    path.display()
                ))
            })
        })
        .collect()
}

pub fn load_history(path: &Path, kind: &str) -> Result<Value, CommandInputJsonError> {
    let text = fs::read_to_string(path)?;
    let history: Value = serde_json::from_str(&text).map_err(|err| {
        CommandInputJsonError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    history
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CommandInputJsonError::Parse(format!(
                "{kind} `{}` must contain an `entries` array",
                path.display()
            ))
        })?;
    Ok(history)
}

pub fn history_entries_mut<'a>(
    history: &'a mut Value,
    path: &Path,
    kind: &str,
) -> Result<&'a mut Vec<Value>, CommandInputJsonError> {
    history
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| {
            CommandInputJsonError::Parse(format!(
                "{kind} `{}` must contain an `entries` array",
                path.display()
            ))
        })
}

pub fn write_json(path: &Path, value: &Value) -> Result<(), CommandInputJsonError> {
    write_json_value(path, value).map_err(Into::into)
}

pub fn read_optional_string(object: &Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

#[macro_export]
macro_rules! define_command_input_json_helpers {
    ($error_type:ident) => {
        #[allow(dead_code)]
        fn load_json_object(
            path: &std::path::Path,
            kind: &str,
        ) -> Result<serde_json::Map<String, serde_json::Value>, $error_type> {
            afterburner::command_input_json::load_json_object(path, kind).map_err(|err| match err {
                afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                    $error_type::Io(err)
                }
                afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                    $error_type::Parse(msg)
                }
            })
        }

        #[allow(dead_code)]
        fn read_string(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
            kind: &str,
        ) -> Result<String, $error_type> {
            afterburner::command_input_json::read_string(object, key, path, kind).map_err(|err| {
                match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                }
            })
        }

        #[allow(dead_code)]
        fn read_u64(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
            kind: &str,
        ) -> Result<u64, $error_type> {
            afterburner::command_input_json::read_u64(object, key, path, kind).map_err(|err| {
                match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                }
            })
        }

        #[allow(dead_code)]
        fn read_array(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
            kind: &str,
        ) -> Result<serde_json::Value, $error_type> {
            afterburner::command_input_json::read_array(object, key, path, kind).map_err(|err| {
                match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                }
            })
        }

        #[allow(dead_code)]
        fn read_object(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
            kind: &str,
        ) -> Result<serde_json::Value, $error_type> {
            afterburner::command_input_json::read_object(object, key, path, kind).map_err(
                |err| match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                },
            )
        }

        #[allow(dead_code)]
        fn read_string_array(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
            kind: &str,
        ) -> Result<Vec<String>, $error_type> {
            afterburner::command_input_json::read_string_array(object, key, path, kind).map_err(
                |err| match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                },
            )
        }

        #[allow(dead_code)]
        fn read_optional_string(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
        ) -> Option<String> {
            afterburner::command_input_json::read_optional_string(object, key)
        }

        #[allow(dead_code)]
        fn write_json(
            path: &std::path::Path,
            value: &serde_json::Value,
            kind: &str,
        ) -> Result<(), $error_type> {
            afterburner::command_input_json::write_json(path, value).map_err(|err| match err {
                afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                    $error_type::Io(err)
                }
                afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                    $error_type::Parse(format!("serialize {kind} json: {msg}"))
                }
            })
        }
    };
}

#[macro_export]
macro_rules! define_command_input_json_kind_helpers {
    ($error_type:ident, $kind:expr) => {
        #[allow(dead_code)]
        fn load_json_object(
            path: &std::path::Path,
            kind: &str,
        ) -> Result<serde_json::Map<String, serde_json::Value>, $error_type> {
            afterburner::command_input_json::load_json_object(path, kind).map_err(|err| match err {
                afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                    $error_type::Io(err)
                }
                afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                    $error_type::Parse(msg)
                }
            })
        }

        #[allow(dead_code)]
        fn read_string(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
        ) -> Result<String, $error_type> {
            afterburner::command_input_json::read_string(object, key, path, $kind).map_err(|err| {
                match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                }
            })
        }

        #[allow(dead_code)]
        fn read_u64(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
        ) -> Result<u64, $error_type> {
            afterburner::command_input_json::read_u64(object, key, path, $kind).map_err(|err| {
                match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                }
            })
        }

        #[allow(dead_code)]
        fn read_array(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
        ) -> Result<serde_json::Value, $error_type> {
            afterburner::command_input_json::read_array(object, key, path, $kind).map_err(|err| {
                match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                }
            })
        }

        #[allow(dead_code)]
        fn read_object(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
        ) -> Result<serde_json::Value, $error_type> {
            afterburner::command_input_json::read_object(object, key, path, $kind).map_err(
                |err| match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                },
            )
        }

        #[allow(dead_code)]
        fn read_string_array(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
        ) -> Result<Vec<String>, $error_type> {
            afterburner::command_input_json::read_string_array(object, key, path, $kind).map_err(
                |err| match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                },
            )
        }

        #[allow(dead_code)]
        fn read_optional_string(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
        ) -> Option<String> {
            afterburner::command_input_json::read_optional_string(object, key)
        }

        #[allow(dead_code)]
        fn write_json(
            path: &std::path::Path,
            value: &serde_json::Value,
            kind: &str,
        ) -> Result<(), $error_type> {
            afterburner::command_input_json::write_json(path, value).map_err(|err| match err {
                afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                    $error_type::Io(err)
                }
                afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                    $error_type::Parse(format!("serialize {kind} json: {msg}"))
                }
            })
        }
    };
}

#[macro_export]
macro_rules! define_command_input_json_kind_history_helpers {
    ($error_type:ident, $kind:expr, $history_kind:expr) => {
        afterburner::define_command_input_json_kind_helpers!($error_type, $kind);

        #[allow(dead_code)]
        fn load_history(path: &std::path::Path) -> Result<serde_json::Value, $error_type> {
            afterburner::command_input_json::load_history(path, $history_kind).map_err(|err| {
                match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                }
            })
        }

        #[allow(dead_code)]
        fn history_entries_mut<'a>(
            history: &'a mut serde_json::Value,
        ) -> Result<&'a mut Vec<serde_json::Value>, $error_type> {
            afterburner::command_input_json::history_entries_mut(
                history,
                std::path::Path::new("history"),
                $history_kind,
            )
            .map_err(|err| match err {
                afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                    $error_type::Io(err)
                }
                afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                    $error_type::Parse(msg)
                }
            })
        }
    };
}

#[macro_export]
macro_rules! define_command_input_json_non_empty_kind_history_helpers {
    ($error_type:ident, $kind:expr, $history_kind:expr) => {
        afterburner::define_command_input_json_non_empty_kind_helpers!($error_type, $kind);

        #[allow(dead_code)]
        fn load_history(path: &std::path::Path) -> Result<serde_json::Value, $error_type> {
            afterburner::command_input_json::load_history(path, $history_kind).map_err(|err| {
                match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                }
            })
        }

        #[allow(dead_code)]
        fn history_entries_mut<'a>(
            history: &'a mut serde_json::Value,
        ) -> Result<&'a mut Vec<serde_json::Value>, $error_type> {
            afterburner::command_input_json::history_entries_mut(
                history,
                std::path::Path::new("history"),
                $history_kind,
            )
            .map_err(|err| match err {
                afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                    $error_type::Io(err)
                }
                afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                    $error_type::Parse(msg)
                }
            })
        }
    };
}

#[macro_export]
macro_rules! define_command_input_json_non_empty_kind_helpers {
    ($error_type:ident, $kind:expr) => {
        #[allow(dead_code)]
        fn load_json_object(
            path: &std::path::Path,
            kind: &str,
        ) -> Result<serde_json::Map<String, serde_json::Value>, $error_type> {
            afterburner::command_input_json::load_json_object(path, kind).map_err(|err| match err {
                afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                    $error_type::Io(err)
                }
                afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                    $error_type::Parse(msg)
                }
            })
        }

        #[allow(dead_code)]
        fn read_string(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
        ) -> Result<String, $error_type> {
            afterburner::command_input_json::read_non_empty_string(object, key, path, $kind)
                .map_err(|err| match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                })
        }

        #[allow(dead_code)]
        fn read_u64(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
        ) -> Result<u64, $error_type> {
            afterburner::command_input_json::read_u64(object, key, path, $kind).map_err(|err| {
                match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                }
            })
        }

        #[allow(dead_code)]
        fn read_array(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
        ) -> Result<serde_json::Value, $error_type> {
            afterburner::command_input_json::read_array(object, key, path, $kind).map_err(|err| {
                match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                }
            })
        }

        #[allow(dead_code)]
        fn read_object(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
        ) -> Result<serde_json::Value, $error_type> {
            afterburner::command_input_json::read_object(object, key, path, $kind).map_err(
                |err| match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                },
            )
        }

        #[allow(dead_code)]
        fn read_string_array(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            path: &std::path::Path,
        ) -> Result<Vec<String>, $error_type> {
            afterburner::command_input_json::read_string_array(object, key, path, $kind).map_err(
                |err| match err {
                    afterburner::command_input_json::CommandInputJsonError::Io(err) => {
                        $error_type::Io(err)
                    }
                    afterburner::command_input_json::CommandInputJsonError::Parse(msg) => {
                        $error_type::Parse(msg)
                    }
                },
            )
        }

        #[allow(dead_code)]
        fn read_optional_string(
            object: &serde_json::Map<String, serde_json::Value>,
            key: &str,
        ) -> Option<String> {
            afterburner::command_input_json::read_optional_string(object, key)
        }
    };
}
