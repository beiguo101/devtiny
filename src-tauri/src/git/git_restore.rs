use crate::core::command_runner::CommandSpec;
use crate::git::git_actions::WorkStep;
use crate::git::git_status::GitFileStatus;

pub fn build_restore_steps(relative_path: &str, status: &GitFileStatus) -> Vec<WorkStep> {
    match status {
        GitFileStatus::Staged => vec![
            WorkStep::Command(CommandSpec::new(
                "git",
                vec![
                    "restore".into(),
                    "--staged".into(),
                    "--".into(),
                    relative_path.into(),
                ],
            )),
            WorkStep::Command(CommandSpec::new(
                "git",
                vec!["restore".into(), "--".into(), relative_path.into()],
            )),
        ],
        GitFileStatus::Untracked => vec![WorkStep::TrashPath {
            relative_path: relative_path.into(),
        }],
        _ => vec![WorkStep::Command(CommandSpec::new(
            "git",
            vec!["restore".into(), "--".into(), relative_path.into()],
        ))],
    }
}
