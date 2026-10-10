//! Project attribution. Never guess from process name alone.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use sweeploom_core::{Confidence, ProcessSnapshot, ProjectAttribution, ProjectId};

/// Known project roots and the "current" project to protect.
#[derive(Clone, Debug, Default)]
pub struct AttributionRoots {
    /// Canonical project directories.
    pub projects: Vec<PathBuf>,
    /// Currently active project, if the user has one.
    pub current_project: Option<ProjectId>,
}

/// Attribute each process to a project using cwd, then ancestor cwd, then command path.
pub fn attribute_projects(processes: &mut [ProcessSnapshot], roots: &AttributionRoots) {
    let projects: Vec<_> = roots
        .projects
        .iter()
        .filter(|p| !crate::is_tool_installation(p))
        .cloned()
        .collect();
    for process in processes.iter_mut() {
        if process.project.is_some() {
            continue;
        }
        if let Some(cwd) = &process.cwd
            && !crate::is_tool_installation(cwd)
            && let Some(project) = containing_project(cwd, &projects)
        {
            process.project = Some(ProjectAttribution {
                project: ProjectId(project),
                confidence: Confidence::Exact,
            });
            continue;
        }
        let script_project = crate::node_identity(process)
            .and_then(|identity| identity.entrypoint)
            .and_then(|path| containing_project(&path, &projects));
        if let Some(project) = script_project.or_else(|| {
            (!crate::is_node(process))
                .then(|| command_contains_project(&process.command, &projects))
                .flatten()
        }) {
            process.project = Some(ProjectAttribution {
                project: ProjectId(project),
                confidence: Confidence::Strong,
            });
        }
    }
    // Inherit a proven parent project only when cwd is unavailable. A helper
    // that changed to another workspace must not inherit the parent's project.
    let by_key: HashMap<_, _> = processes
        .iter()
        .map(|p| (p.key, (p.parent, p.project.clone())))
        .collect();
    for process in processes
        .iter_mut()
        .filter(|p| p.project.is_none() && p.cwd.is_none())
    {
        let mut parent = process.parent;
        let mut visited = HashSet::new();
        while let Some(key) = parent {
            if !visited.insert(key) {
                break;
            }
            let Some((ancestor, project)) = by_key.get(&key) else {
                break;
            };
            if let Some(project) = project {
                process.project = Some(ProjectAttribution {
                    project: project.project.clone(),
                    confidence: Confidence::Strong,
                });
                break;
            }
            parent = *ancestor;
        }
    }
}

fn containing_project(path: &Path, projects: &[PathBuf]) -> Option<PathBuf> {
    projects
        .iter()
        .filter(|project| path.starts_with(project))
        .max_by_key(|project| project.components().count())
        .cloned()
}

fn command_contains_project(command: &[String], projects: &[PathBuf]) -> Option<PathBuf> {
    // argv[0] identifies the installed runtime; it is not a workspace signal.
    for token in command
        .iter()
        .skip(1)
        .filter(|s| !s.starts_with('-') && !s.contains('=') && !s.contains("://"))
    {
        let path = Path::new(token);
        if let Some(project) = containing_project(path, projects) {
            return Some(project);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::proc;

    #[test]
    fn node_installation_is_never_the_project() {
        let mut processes = vec![proc(
            7,
            None,
            "node",
            Some("/work/demo"),
            &[
                "/home/me/.nvm/versions/node/v22.13.1/bin/node",
                "/work/demo/server.js",
            ],
            1,
            0.0,
        )];
        let roots = AttributionRoots {
            projects: vec!["/home/me/.nvm".into(), "/work/demo".into()],
            current_project: None,
        };
        attribute_projects(&mut processes, &roots);
        assert_eq!(
            processes[0].project.as_ref().unwrap().project.0,
            PathBuf::from("/work/demo")
        );
        processes[0].project = None;
        processes[0].cwd = None;
        processes[0].command = vec!["/home/me/.nvm/versions/node/v22.13.1/bin/node".into()];
        attribute_projects(&mut processes, &roots);
        assert_eq!(processes[0].project, None);
    }

    #[test]
    fn helper_inherits_proven_ancestor_only_when_cwd_missing() {
        let mut processes = vec![
            proc(10, None, "claude", Some("/work/demo"), &["claude"], 1, 0.0),
            proc(11, Some(10), "node", None, &["node"], 1, 0.0),
            proc(12, Some(10), "node", Some("/elsewhere"), &["node"], 1, 0.0),
        ];
        attribute_projects(
            &mut processes,
            &AttributionRoots {
                projects: vec!["/work/demo".into()],
                current_project: None,
            },
        );
        assert_eq!(
            processes[1].project.as_ref().unwrap().project.0,
            PathBuf::from("/work/demo")
        );
        assert_eq!(processes[2].project, None);
    }
}
