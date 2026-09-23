pub mod action;
pub mod capability;
pub mod config;
pub mod decision;
pub mod event;
mod http;
pub mod model;
pub mod runtime;
pub use capability::WorkspaceFs;

#[cfg(test)]
mod tests {
    use super::capability::{CapabilityRegistry, WorkspaceFs};
    use std::fs;

    #[test]
    fn unknown_capabilities_are_rejected_and_empty_is_deny_by_default() {
        let empty = CapabilityRegistry::from_names(&[] as &[&str]).unwrap();
        assert!(empty.is_empty());
        assert!(CapabilityRegistry::from_names(&["unknown"]).is_err());
    }

    #[test]
    fn manifest_rejects_inline_auth_and_loads_env_based_models() {
        let inline = "name: bad\nauth: plaintext\n";
        assert!(crate::config::Manifest::from_yaml(inline).is_err());

        let safe = r#"
name: safe
workspace: .
capabilities: [read_file]
models:
  - provider: dmr-x
    model: auto
    endpoint: http://127.0.0.1:47113/v1/chat/completions
    auth_env: DMRX_API_KEY
"#;
        let manifest = crate::config::Manifest::from_yaml(safe).unwrap();
        assert_eq!(manifest.models.len(), 1);
        assert_eq!(manifest.models[0].auth_env.as_deref(), Some("DMRX_API_KEY"));
        assert!(!manifest.unsafe_shell_enabled());
    }

    #[test]
    fn per_tool_limit_defaults_to_five_and_can_be_configured() {
        let default = crate::config::Manifest::from_yaml("name: default\n").unwrap();
        assert_eq!(default.max_per_tool, 5);
        let custom = crate::config::Manifest::from_yaml("name: custom\nmax_per_tool: 2\n").unwrap();
        assert_eq!(custom.max_per_tool, 2);
    }

    #[test]
    fn every_example_manifest_is_safe_and_valid() {
        let specs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("specs");
        let mut parsed = 0;
        for entry in std::fs::read_dir(specs).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|value| value.to_str()) != Some("yaml") {
                continue;
            }
            let yaml = std::fs::read_to_string(&path).unwrap();
            assert!(!yaml
                .lines()
                .any(|line| line.trim_start().starts_with("auth:")));
            let manifest = crate::config::Manifest::from_yaml(&yaml)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let workspace = WorkspaceFs::new(manifest.workspace_from(&path)).unwrap();
            crate::runtime::Runtime::new(&manifest, workspace)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            parsed += 1;
        }
        assert!(parsed >= 14);
    }

    #[test]
    fn workspace_fs_rejects_parent_absolute_and_symlink_escape() {
        let root = std::env::temp_dir().join(format!("mote-fs-{}", std::process::id()));
        let outside = root.with_extension("outside");
        let missing = root.with_extension("missing");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_file(&outside);
        let _ = fs::remove_file(&missing);
        fs::create_dir_all(&root).unwrap();
        fs::write(&outside, "secret").unwrap();
        let link = root.join("link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        #[cfg(windows)]
        if std::os::windows::fs::symlink_file(&outside, &link).is_err() {
            let _ = fs::remove_dir_all(&root);
            let _ = fs::remove_file(&outside);
            return;
        }
        let fsx = WorkspaceFs::new(&root).unwrap();
        assert!(fsx.read("../mote-fs-outside").is_err());
        assert!(fsx.read(outside.to_str().unwrap()).is_err());
        assert!(fsx.read("link").is_err());
        assert!(fsx.write("link", "x").is_err());
        assert!(fsx.list("link").is_err());
        let dangling = root.join("dangling");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&missing, &dangling).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&missing, &dangling).unwrap();
        assert!(fsx.write("dangling", "should not escape").is_err());
        assert!(!missing.exists());
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_file(&outside);
    }

    #[test]
    fn git_capability_without_shell_authorization_is_rejected() {
        let manifest =
            crate::config::Manifest::from_yaml("name: git-only\ncapabilities: [git]\nmodels: []\n")
                .unwrap();
        let workspace = WorkspaceFs::new(std::env::current_dir().unwrap()).unwrap();
        assert!(crate::runtime::Runtime::new(&manifest, workspace).is_err());
        let manifest = crate::config::Manifest::from_yaml(
            "name: git-not-allowlisted\ncapabilities: [git, shell]\nshell:\n  allowed_programs: [rustc]\n",
        )
        .unwrap();
        let workspace = WorkspaceFs::new(std::env::current_dir().unwrap()).unwrap();
        assert!(crate::runtime::Runtime::new(&manifest, workspace).is_err());
    }

    #[test]
    fn shell_requires_allowlisted_executable_and_rejects_operators() {
        use crate::runtime::{CancellationToken, ShellExecutor};
        use std::collections::BTreeSet;
        use std::time::Duration;
        let executor = ShellExecutor {
            workspace: std::env::current_dir().unwrap(),
            allowed: BTreeSet::new(),
            timeout: Duration::from_millis(50),
            max_output: 32,
            unsafe_shell: false,
        };
        assert!(executor
            .run("echo safe", &CancellationToken::default())
            .is_err());
        let mut allowed = BTreeSet::new();
        allowed.insert("echo".into());
        let executor = ShellExecutor {
            allowed,
            ..executor
        };
        assert!(executor
            .run("echo safe && echo unsafe", &CancellationToken::default())
            .is_err());
        let path_alias = ShellExecutor {
            allowed: BTreeSet::from(["mote-nonexistent".to_owned()]),
            ..executor
        };
        assert_eq!(
            path_alias
                .run("elsewhere/mote-nonexistent", &CancellationToken::default())
                .unwrap_err(),
            "executable is not allowlisted"
        );
    }

    #[test]
    fn safe_git_rejects_inline_alias_and_external_diff() {
        use crate::runtime::{CancellationToken, ShellExecutor};
        use std::collections::BTreeSet;
        use std::time::Duration;
        let executor = ShellExecutor {
            workspace: std::env::current_dir().unwrap(),
            allowed: BTreeSet::from(["git".to_owned()]),
            timeout: Duration::from_secs(2),
            max_output: 256,
            unsafe_shell: false,
        };
        assert!(executor
            .run("git status --short", &CancellationToken::default())
            .is_ok());
        assert_eq!(
            executor
                .run(
                    "git -c alias.probe=!echo probe",
                    &CancellationToken::default()
                )
                .unwrap_err(),
            "git subcommand not permitted"
        );
        assert_eq!(
            executor
                .run("git diff --ext-diff", &CancellationToken::default())
                .unwrap_err(),
            "git option not permitted"
        );
        for command in [
            "git log --help",
            "git show -h",
            "git diff --no-ext-diff --help",
        ] {
            assert_eq!(
                executor
                    .run(command, &CancellationToken::default())
                    .unwrap_err(),
                "git option not permitted",
                "{command} must never launch a help viewer"
            );
        }
        assert!(executor
            .run("git log -1 --oneline", &CancellationToken::default())
            .is_ok());
    }

    #[test]
    fn safe_git_ignores_local_signature_helper_configuration() {
        use crate::runtime::{CancellationToken, ShellExecutor};
        use std::collections::BTreeSet;
        use std::io::Write;
        use std::process::{Command, Stdio};
        use std::time::Duration;
        let root = std::env::temp_dir().join(format!("mote-git-signature-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&root)
                .args(args)
                .output()
                .unwrap()
        };
        assert!(git(&["init", "-q"]).status.success());
        assert!(git(&["symbolic-ref", "HEAD", "refs/heads/master"])
            .status
            .success());
        let tree = {
            let process = Command::new("git")
                .current_dir(&root)
                .args(["hash-object", "-t", "tree", "-w", "--stdin"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            String::from_utf8(process.wait_with_output().unwrap().stdout)
                .unwrap()
                .trim()
                .to_owned()
        };
        let raw = format!(
            "tree {tree}\nauthor Probe <probe@example.org> 1780000000 +0000\ncommitter Probe <probe@example.org> 1780000000 +0000\ngpgsig -----BEGIN PGP SIGNATURE-----\n YWJj\n -----END PGP SIGNATURE-----\n\nprobe\n"
        );
        let mut process = Command::new("git")
            .current_dir(&root)
            .args(["hash-object", "-t", "commit", "-w", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        process
            .stdin
            .take()
            .unwrap()
            .write_all(raw.as_bytes())
            .unwrap();
        let result = process.wait_with_output().unwrap();
        assert!(result.status.success());
        let commit = String::from_utf8(result.stdout).unwrap();
        assert!(git(&["update-ref", "refs/heads/master", commit.trim()])
            .status
            .success());
        assert!(git(&["config", "log.showSignature", "true"])
            .status
            .success());
        assert!(git(&["config", "gpg.program", "mote-nonexistent-gpg"])
            .status
            .success());
        let executor = ShellExecutor {
            workspace: root.clone(),
            allowed: BTreeSet::from(["git".to_owned()]),
            timeout: Duration::from_secs(2),
            max_output: 4096,
            unsafe_shell: false,
        };
        let result = executor.run("git log -1 --oneline", &CancellationToken::default());
        assert!(result.is_ok(), "{result:?}");
        assert!(!result.unwrap().contains("mote-nonexistent-gpg"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn shell_reports_nonzero_timeout_cancellation_and_caps_output() {
        use crate::runtime::{CancellationToken, ShellExecutor};
        use std::collections::BTreeSet;
        use std::time::{Duration, Instant};

        let workspace = std::env::current_dir().unwrap();
        let git = ShellExecutor {
            workspace: workspace.clone(),
            allowed: BTreeSet::from(["git".to_owned()]),
            timeout: Duration::from_secs(2),
            max_output: 128,
            unsafe_shell: false,
        };
        assert!(git
            .run(
                "git rev-parse refs/heads/__mote_missing_branch__",
                &CancellationToken::default()
            )
            .unwrap_err()
            .contains("command failed"));

        let cancelled = CancellationToken::default();
        cancelled.cancel();
        assert_eq!(
            git.run("git --version", &cancelled).unwrap_err(),
            "cancelled"
        );

        let capped = ShellExecutor {
            allowed: BTreeSet::from(["rustc".to_owned()]),
            max_output: 4,
            ..git
        };
        assert_eq!(
            capped
                .run("rustc --version", &CancellationToken::default())
                .unwrap()
                .len(),
            4
        );
        let quoted = ShellExecutor {
            allowed: BTreeSet::from(["rustc".to_owned()]),
            max_output: 4096,
            ..capped
        };
        assert!(quoted
            .run(r#"rustc --print "cfg""#, &CancellationToken::default())
            .unwrap()
            .contains("target_arch"));

        #[cfg(windows)]
        let slow = ("ping", "ping -n 4 127.0.0.1");
        #[cfg(not(windows))]
        let slow = ("sleep", "sleep 2");
        let timed = ShellExecutor {
            allowed: BTreeSet::from([slow.0.to_owned()]),
            timeout: Duration::from_millis(50),
            max_output: 128,
            workspace,
            unsafe_shell: false,
        };
        let started = Instant::now();
        assert_eq!(
            timed
                .run(slow.1, &CancellationToken::default())
                .unwrap_err(),
            "command timed out"
        );
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn decision_fallback_is_typed_and_deterministic() {
        use crate::decision::{DecisionAnswer, DecisionQuestion, DecisionRequest};
        use std::collections::BTreeMap;
        let request = DecisionRequest {
            id: "gate".to_owned(),
            state: serde_json::json!("state"),
            question: DecisionQuestion::Choice {
                instructions: "Choose".to_owned(),
                criteria: BTreeMap::from([
                    ("allow".to_owned(), "allow".to_owned()),
                    ("deny".to_owned(), "deny".to_owned()),
                ]),
            },
        };
        assert!(matches!(
            crate::decision::deterministic_fallback(&request, None),
            DecisionAnswer::Choice { ref choice, .. } if choice == "deny"
        ));
    }

    #[test]
    fn runtime_stops_repeated_actions_at_iteration_limit() {
        use crate::decision::{DecisionQuestion, DecisionRequest};
        use std::collections::BTreeMap;
        let root = std::env::current_dir().unwrap();
        let fsx = WorkspaceFs::new(root).unwrap();
        let manifest = crate::config::Manifest::from_yaml(
            "name: test\ncapabilities: [decision]\nmax_iterations: 2\n",
        )
        .unwrap();
        let mut runtime = crate::runtime::Runtime::new(&manifest, fsx).unwrap();
        let action = crate::action::Action::Decision {
            request: DecisionRequest {
                id: "q".to_owned(),
                state: serde_json::json!("state"),
                question: DecisionQuestion::Choice {
                    instructions: "Choose".to_owned(),
                    criteria: BTreeMap::from([("a".to_owned(), "a".to_owned())]),
                },
            },
        };
        assert_eq!(
            runtime.run_actions(&[action.clone(), action.clone(), action]),
            crate::runtime::State::Failed
        );
    }
}
