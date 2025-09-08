/// Permissions rationale for path validation
pub struct PermissionsRationale;
#[allow(dead_code)]
impl PermissionsRationale {
    pub fn explain() -> &'static str {
        "Permissions rationale:\n\
        1. Only files under `allowed_paths` are accessible to avoid arbitrary filesystem access.\n\
        2. File extensions are restricted to prevent execution of unsafe files.\n\
        3. Path length and prohibited characters prevent injection or path traversal attacks.\n\
        Review whenever new file operations are added."
    }
}
