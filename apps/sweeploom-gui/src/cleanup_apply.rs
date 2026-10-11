use super::*;

pub(super) fn cache_candidate(item: &Item, index: usize) -> Option<sweeploom_core::Candidate> {
    use sweeploom_core::*;
    if item.kind != Kind::Cache || !item.usage.is_some_and(|usage| usage.complete) {
        return None;
    }
    Some(Candidate {
        id: CandidateId(900_000 + index as u64),
        kind: CandidateKind::BuildArtifact,
        owner: CandidateOwner::User,
        path: item.path.clone()?,
        logical_bytes: item.usage.map(|usage| usage.logical_bytes).unwrap_or(0),
        allocated_bytes: item.bytes,
        file_count: item.usage.map(|usage| usage.files).unwrap_or(0),
        activity: ActivityEvidence::default(),
        safety: SafetyAssessment::review(),
        rebuild: RebuildAssessment::default(),
        deletion: DeletionStrategy::PermanentGenerated,
        evidence: vec![],
        user_policy: UserPolicy::AskEveryTime,
    })
}
pub(super) fn apply_native(
    items: &[Item],
    processes: &[sweeploom_core::ProcessSnapshot],
) -> String {
    let mut done = 0;
    let mut errors = Vec::new();
    for item in items {
        if !valid_id(&item.id)
            && !(item.kind == Kind::Model
                && !item.id.starts_with('-')
                && item.id.chars().all(|ch| {
                    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':' | '/')
                }))
        {
            errors.push(format!("Invalid object id: {}", item.id));
            continue;
        }
        let result = match item.kind {
            Kind::Model => models::remove(item),
            Kind::Toolchain => toolchains::remove(item, processes),
            Kind::Image => docker_command(item, &["image", "rm", &item.id]),
            Kind::Container => docker_command(item, &["container", "rm", &item.id]),
            Kind::Volume => docker_command(item, &["volume", "rm", &item.id]),
            Kind::BuildCache => docker_command(
                item,
                &["builder", "prune", "--filter", "until=168h", "--force"],
            ),
            Kind::Device => ios::remove_device(item),
            Kind::SimUnavailable => ios::remove_unavailable(item),
            Kind::SimFiles => ios::remove_files(item, processes),
            Kind::Runtime => ios::list(None).and_then(|value| {
                if parse_devices(&value).iter().any(|device| !device.enabled) {
                    return Err("A simulator is running; stop it before removing a runtime.".into());
                }
                ios::simctl(
                    None,
                    &["runtime", "delete", &item.id],
                    std::time::Duration::from_secs(300),
                )
            }),
            _ => Err("Use file cleanup for this item.".into()),
        };
        match result {
            Ok(_) => done += 1,
            Err(error) => errors.push(format!("{}: {error}", item.name)),
        }
    }
    format!("Cleaned {done} object(s). {}", errors.join("\n"))
}
