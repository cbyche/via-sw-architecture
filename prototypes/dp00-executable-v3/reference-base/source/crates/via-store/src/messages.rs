//! The persistence warning strings.
//!
//! Every string here is an **external contract**: it is surfaced verbatim on
//! `/api/health` as `<surface>.warning.message` and logged as
//! `<surface>.persistence_warning`, so a human reading a health payload from a
//! Node Gateway and from VIA must see the same sentence.
//!
//! Upstream sources:
//! - [`StoreMessages::DEFAULT`] — `server/src/core/versioned-json-store.mjs`
//!   lines 28-73 (all six strings).
//! - [`StoreMessages::TASK_STORE`] — `server/src/task/task-store.mjs`
//!   lines 45-104, catalogued as contract *"task store quarantine path +
//!   warnings"*.
//!
//! The two sets differ in four of the six slots, which is why the string table
//! is a parameter of the store rather than baked into it.

/// The six warning strings a [`VersionedJsonStore`] can emit.
///
/// Held as plain function pointers so a surface (task store, notes store, ACP
/// session index, …) can supply its own catalogued wording without the store
/// growing a trait object per message.
///
/// [`VersionedJsonStore`]: crate::VersionedJsonStore
#[derive(Debug, Clone, Copy)]
pub struct StoreMessages {
    /// The document did not parse as JSON. Arguments: `label`, engine message.
    pub invalid_json: fn(&str, &str) -> String,
    /// The document parsed but its `version` or shape was rejected. Argument:
    /// `label`.
    pub invalid_shape: fn(&str) -> String,
    /// The document could not be read for a reason other than "absent".
    /// Arguments: `label`, engine message.
    pub read_failed: fn(&str, &str) -> String,
    /// The document could not be written. Arguments: `label`, engine message.
    pub save_failed: fn(&str, &str) -> String,
    /// The original was successfully moved aside. Arguments: the reason string
    /// produced by one of the three above, the quarantine path.
    pub quarantined: fn(&str, &str) -> String,
    /// Quarantine itself failed, so persistence is now disabled. Arguments: the
    /// reason string, the engine message from the failed rename.
    pub quarantine_failed: fn(&str, &str) -> String,
}

impl StoreMessages {
    /// The generic [`VersionedJsonStore`] wording.
    ///
    /// Contract — `server/src/core/versioned-json-store.mjs`:
    /// - `${label}文件不是有效的 JSON：${error.message}` (line 60)
    /// - `${label}文件格式或版本无效` (line 68)
    /// - `无法读取${label}文件：${error.message}` (line 62)
    /// - `无法保存${label}文件：${error.message}` (line 86)
    /// - `${reason}；原文件已隔离为 ${quarantinePath}。` (lines 36-39)
    /// - `${reason}；隔离失败（${error.message}），已禁用持久化以保护原文件。` (lines 41-45)
    ///
    /// [`VersionedJsonStore`]: crate::VersionedJsonStore
    pub const DEFAULT: Self = Self {
        invalid_json: |label, detail| format!("{label}文件不是有效的 JSON：{detail}"),
        invalid_shape: |label| format!("{label}文件格式或版本无效"),
        read_failed: |label, detail| format!("无法读取{label}文件：{detail}"),
        save_failed: |label, detail| format!("无法保存{label}文件：{detail}"),
        quarantined: |reason, path| format!("{reason}；原文件已隔离为 {path}。"),
        quarantine_failed: |reason, detail| {
            format!("{reason}；隔离失败（{detail}），已禁用持久化以保护原文件。")
        },
    };

    /// The `tasks.json` wording, which upstream hand-wrote rather than deriving
    /// from the label.
    ///
    /// Contract — `server/src/task/task-store.mjs` lines 45-104, catalogued as
    /// *"task store quarantine path + warnings"*. Use with
    /// [`label`](crate::VersionedJsonStoreBuilder::label) `"任务状态"`: two of the
    /// six still interpolate it, and the other four are fixed sentences that
    /// name the task surface explicitly ("空任务状态", "任务持久化").
    pub const TASK_STORE: Self = Self {
        invalid_json: |label, detail| format!("{label}文件不是有效的 JSON：{detail}"),
        invalid_shape: |label| format!("{label}文件格式无效"),
        read_failed: |label, detail| format!("无法读取{label}文件：{detail}"),
        save_failed: |label, detail| format!("无法保存{label}：{detail}"),
        quarantined: |reason, path| {
            format!("{reason}；原文件已隔离为 {path}，服务将使用空任务状态继续运行。")
        },
        quarantine_failed: |reason, detail| {
            format!("{reason}；隔离失败（{detail}），已禁用任务持久化以保护原文件。")
        },
    };
}

impl Default for StoreMessages {
    fn default() -> Self {
        Self::DEFAULT
    }
}
