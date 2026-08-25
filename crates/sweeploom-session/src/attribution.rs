//! Project attribution. Never guess from process name alone.

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
    if roots.projects.is_empty() {
        return;
    }
    for process in processes.iter_mut() {
        if process.project.is_some() {
            continue;
        }
        if let Some(cwd) = &process.cwd
            && let Some(project) = containing_project(cwd, &roots.projects)
        {
            process.project = Some(ProjectAttribution {
                project: ProjectId(project),
                confidence: Confidence::Exact,
            });
            continue;
        }
        if let Some(project) = command_contains_project(&process.command, &roots.projects) {
            process.project = Some(ProjectAttribution {
                project: ProjectId(project),
                confidence: Confidence::Strong,
            });
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
    for token in command {
        let path = Path::new(token);
        if let Some(project) = containing_project(path, projects) {
            return Some(project);
        }
    }
    None
}
