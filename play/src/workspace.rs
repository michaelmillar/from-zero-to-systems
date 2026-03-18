use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{lang_runner, meta, meta::CRATES, progress};
const LOCAL_WORKSPACE_REL: &str = ".fzts/workspace";
const LOCAL_ORIGIN_FILE: &str = ".fzts-origin";
const LOCAL_SEED_ENTRIES: &[&str] = &["Cargo.toml", "Cargo.lock", "crates", "play"];

type DynError = Box<dyn std::error::Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeRecord {
    pub id: String,
    pub title: String,
    pub package: String,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct DocLinkRecord {
    pub(crate) label: String,
    pub(crate) url: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HintRecord {
    pub(crate) test_name: String,
    pub(crate) hints: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceRecord {
    pub(crate) challenge: String,
    pub(crate) title: String,
    pub(crate) file_path: String,
    pub(crate) content: String,
    pub(crate) intro: String,
    pub(crate) guide: String,
    pub(crate) concepts: Vec<String>,
    pub(crate) docs: Vec<DocLinkRecord>,
    pub(crate) hints: Vec<HintRecord>,
    pub(crate) can_reset: bool,
}

pub(crate) use lang_runner::TestResponse;

pub fn resolve_workspace_root(current_dir: &Path, explicit: Option<&Path>) -> PathBuf {
    if let Some(explicit) = explicit {
        return explicit.to_path_buf();
    }

    let local = local_workspace_root(current_dir);
    if local.join("Cargo.toml").exists() {
        return local;
    }

    let file_name = match current_dir.file_name().and_then(|name| name.to_str()) {
        Some(name) => name,
        None => return current_dir.to_path_buf(),
    };

    if file_name.ends_with("-challenges") {
        return current_dir.to_path_buf();
    }

    let parent = match current_dir.parent() {
        Some(parent) => parent,
        None => return current_dir.to_path_buf(),
    };

    let candidate_name = match file_name.strip_suffix("-solutions") {
        Some(base) => format!("{base}-challenges"),
        None => format!("{file_name}-challenges"),
    };
    let candidate = parent.join(candidate_name);

    if candidate.is_dir() {
        candidate
    } else {
        current_dir.to_path_buf()
    }
}

pub fn ensure_local_workspace(current_dir: &Path) -> Result<PathBuf, DynError> {
    let local = local_workspace_root(current_dir);
    if local.join("Cargo.toml").exists() {
        return Ok(local);
    }

    let seed = seed_workspace_root(current_dir).ok_or_else(|| {
        format!(
            "no local seed workspace found. expected a sibling `from-zero-to-systems-challenges` worktree near {}",
            current_dir.display()
        )
    })?;

    if let Some(parent) = local.parent() {
        fs::create_dir_all(parent)?;
    }

    for entry in LOCAL_SEED_ENTRIES {
        let source = seed.join(entry);
        if !source.exists() {
            continue;
        }
        copy_entry(&source, &local.join(entry))?;
    }

    fs::write(local.join(LOCAL_ORIGIN_FILE), seed.display().to_string())?;
    Ok(local)
}

pub fn discover_challenges(workspace: &Path) -> io::Result<Vec<ChallengeRecord>> {
    let crates_dir = workspace.join("crates");
    if !crates_dir.exists() {
        return Ok(Vec::new());
    }

    let progress = progress::load(workspace);
    Ok(sorted_dirs(&crates_dir)?
        .into_iter()
        .filter(|entry| entry.path().join("src/lib.rs").exists())
        .map(|entry| {
            let id = entry.file_name().to_string_lossy().to_string();
            let package = package_from_id(&id);
            let title = title_for(&id, &package);
            let completed = progress.is_completed(package.as_str());

            ChallengeRecord {
                id,
                title,
                package,
                completed,
            }
        })
        .collect())
}

pub(crate) fn load_workspace_for_language(
    workspace: &Path,
    challenge: &str,
    language: &str,
) -> Result<WorkspaceRecord, DynError> {
    let record = challenge_record(workspace, challenge)?
        .ok_or_else(|| format!("unknown challenge: {challenge}"))?;
    let rel_path = challenge_source_rel_path_for_language(challenge, language);
    let file_path = workspace.join(&rel_path);
    let content = fs::read_to_string(&file_path)
        .map_err(|err| format!("failed to read {}: {err}", file_path.display()))?;
    let guide = read_guide(workspace, challenge)?;
    let can_reset = tracked_file_content(workspace, &rel_path).is_ok();

    let (intro, concepts, docs, hints) =
        if let Some(toml_meta) = meta::load_challenge_meta(workspace, challenge) {
            (
                toml_meta.shared.intro.clone(),
                meta::merged_concepts(&toml_meta, language),
                meta::merged_docs(&toml_meta, language)
                    .into_iter()
                    .map(|d| DocLinkRecord {
                        label: d.label,
                        url: d.url,
                    })
                    .collect(),
                meta::merged_hints(&toml_meta, language)
                    .into_iter()
                    .map(|h| HintRecord {
                        test_name: h.test_name,
                        hints: h.hints,
                    })
                    .collect(),
            )
        } else {
            let static_meta = CRATES.iter().find(|m| m.package == record.package);
            (
                static_meta
                    .map(|m| m.intro.to_string())
                    .unwrap_or_default(),
                static_meta
                    .map(|m| m.concepts.iter().map(|c| c.to_string()).collect())
                    .unwrap_or_default(),
                static_meta
                    .map(|m| {
                        m.docs
                            .iter()
                            .map(|d| DocLinkRecord {
                                label: d.label.to_string(),
                                url: d.url.to_string(),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                static_meta
                    .map(|m| {
                        m.tests
                            .iter()
                            .map(|h| HintRecord {
                                test_name: h.test_name.to_string(),
                                hints: h.hints.iter().map(|s| s.to_string()).collect(),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            )
        };

    Ok(WorkspaceRecord {
        challenge: record.id,
        title: record.title,
        file_path: rel_path,
        content,
        intro,
        guide,
        concepts,
        docs,
        hints,
        can_reset,
    })
}

pub(crate) fn save_workspace_for_language(
    workspace: &Path,
    challenge: &str,
    language: &str,
    content: &str,
) -> Result<WorkspaceRecord, DynError> {
    let rel_path = challenge_source_rel_path_for_language(challenge, language);
    let path = workspace.join(&rel_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)
        .map_err(|err| format!("failed to write {}: {err}", path.display()))?;

    load_workspace_for_language(workspace, challenge, language)
}

pub(crate) fn reset_workspace_for_language(
    workspace: &Path,
    challenge: &str,
    language: &str,
) -> Result<WorkspaceRecord, DynError> {
    let rel_path = challenge_source_rel_path_for_language(challenge, language);
    let tracked = tracked_file_content(workspace, &rel_path)?;
    save_workspace_for_language(workspace, challenge, language, &tracked)
}

pub(crate) fn run_workspace_tests_for_language(
    workspace: &Path,
    challenge: &str,
    language: &str,
    content: &str,
) -> Result<TestResponse, DynError> {
    let record = challenge_record(workspace, challenge)?
        .ok_or_else(|| format!("unknown challenge: {challenge}"))?;
    save_workspace_for_language(workspace, challenge, language, content)?;
    progress::record_check_in(workspace);

    let runner = lang_runner::runner_for(language);
    let result = runner.run_tests(workspace, challenge, &record.package)?;

    if result.passed {
        mark_completed_for_language(workspace, &record.package, language);
    }

    Ok(result)
}

fn challenge_record(
    workspace: &Path,
    challenge: &str,
) -> Result<Option<ChallengeRecord>, DynError> {
    Ok(discover_challenges(workspace)?
        .into_iter()
        .find(|record| record.id == challenge))
}

fn mark_completed_for_language(workspace: &Path, package: &str, language: &str) {
    let _ = progress::record_completion_for_language(workspace, package, language);
}

fn challenge_source_rel_path_for_language(challenge: &str, language: &str) -> String {
    lang_runner::runner_for(language).source_rel_path(challenge)
}

pub(crate) fn available_languages(workspace: &Path, challenge: &str) -> Vec<String> {
    let challenge_dir = workspace.join("crates").join(challenge);
    let runners: Vec<Box<dyn lang_runner::LanguageRunner>> = vec![
        lang_runner::runner_for("rust"),
        lang_runner::runner_for("c"),
        lang_runner::runner_for("python"),
        lang_runner::runner_for("haskell"),
    ];

    runners
        .into_iter()
        .filter(|r| r.is_available(&challenge_dir) && r.toolchain_present())
        .map(|r| r.id().to_string())
        .collect()
}

fn tracked_file_content(workspace: &Path, rel_path: &str) -> Result<String, DynError> {
    if let Some(seed_root) = local_seed_origin(workspace) {
        let seed_path = seed_root.join(rel_path);
        return fs::read_to_string(&seed_path).map_err(|err| {
            format!("failed to read seed file {}: {err}", seed_path.display()).into()
        });
    }

    let output = Command::new("git")
        .current_dir(workspace)
        .arg("show")
        .arg(format!("HEAD:{rel_path}"))
        .output()
        .map_err(|err| format!("failed to read tracked file {rel_path}: {err}"))?;

    if !output.status.success() {
        return Err(format!("unable to reset {rel_path} from git HEAD").into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn read_guide(workspace: &Path, challenge: &str) -> Result<String, DynError> {
    let path = workspace.join("crates").join(challenge).join("README.md");
    if !path.exists() {
        return Ok(String::new());
    }

    let raw = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok(strip_markdown(&raw))
}

fn strip_markdown(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            line.trim_start_matches('#')
                .trim_start_matches('-')
                .trim_start()
                .replace("**", "")
                .replace('`', "")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}


fn title_for(id: &str, package: &str) -> String {
    if let Some(meta) = CRATES.iter().find(|meta| meta.package == package) {
        return meta.display.to_string();
    }

    match id.split_once('-') {
        Some((prefix, _)) if prefix.chars().all(|ch| ch.is_ascii_digit()) => {
            format!("{prefix} · {package}")
        }
        _ => humanize_id(id),
    }
}

fn humanize_id(id: &str) -> String {
    id.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn package_from_id(id: &str) -> String {
    id.split_once('-')
        .map(|(_, package)| package.to_string())
        .unwrap_or_else(|| id.to_string())
}

fn local_workspace_root(current_dir: &Path) -> PathBuf {
    current_dir.join(LOCAL_WORKSPACE_REL)
}

fn seed_workspace_root(current_dir: &Path) -> Option<PathBuf> {
    if current_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with("-challenges"))
        && current_dir.join("Cargo.toml").exists()
        && current_dir.join("crates").is_dir()
    {
        return Some(current_dir.to_path_buf());
    }

    let sibling = sibling_challenges_root(current_dir)?;
    if sibling.join("Cargo.toml").exists() && sibling.join("crates").is_dir() {
        Some(sibling)
    } else {
        None
    }
}

fn sibling_challenges_root(current_dir: &Path) -> Option<PathBuf> {
    let file_name = current_dir.file_name()?.to_str()?;
    let parent = current_dir.parent()?;
    let candidate_name = match file_name.strip_suffix("-solutions") {
        Some(base) => format!("{base}-challenges"),
        None => format!("{file_name}-challenges"),
    };
    Some(parent.join(candidate_name))
}

fn local_seed_origin(workspace: &Path) -> Option<PathBuf> {
    let origin_path = workspace.join(LOCAL_ORIGIN_FILE);
    let raw = fs::read_to_string(origin_path).ok()?;
    let seed = PathBuf::from(raw.trim());
    seed.exists().then_some(seed)
}

fn copy_entry(source: &Path, dest: &Path) -> Result<(), DynError> {
    if source.is_dir() {
        copy_dir(source, dest)
    } else {
        copy_file(source, dest)
    }
}

fn copy_dir(source: &Path, dest: &Path) -> Result<(), DynError> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir(&source_path, &dest_path)?;
        } else {
            copy_file(&source_path, &dest_path)?;
        }
    }

    Ok(())
}

fn copy_file(source: &Path, dest: &Path) -> Result<(), DynError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, dest)?;
    Ok(())
}

fn sorted_dirs(path: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ty| ty.is_dir()).unwrap_or(false))
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "play-workspace-{prefix}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    fn ensure_dir(path: &Path) {
        fs::create_dir_all(path).expect("directory should be created");
    }

    #[test]
    fn title_for_prefers_metadata_display() {
        assert_eq!(
            title_for("01-risk-sampler", "risk-sampler"),
            "01 · risk-sampler"
        );
    }

    #[test]
    fn resolve_workspace_root_prefers_sibling_challenges_worktree() {
        let root = temp_dir("resolve");
        let repo = root.join("from-zero-to-systems");
        let challenges = root.join("from-zero-to-systems-challenges");
        ensure_dir(&repo);
        ensure_dir(&challenges);

        let resolved = resolve_workspace_root(&repo, None);

        assert_eq!(resolved, challenges);

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }

    #[test]
    fn resolve_workspace_root_prefers_existing_local_fzts_workspace() {
        let root = temp_dir("local-root");
        let repo = root.join("from-zero-to-systems");
        let local = repo.join(".fzts/workspace");
        ensure_dir(&local);
        fs::write(local.join("Cargo.toml"), "[workspace]\n")
            .expect("local workspace manifest should exist");

        let resolved = resolve_workspace_root(&repo, None);

        assert_eq!(resolved, local);

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }

    #[test]
    fn discover_challenges_reads_and_sorts_numbered_crates() {
        let root = temp_dir("discover");
        ensure_dir(&root.join("crates/02-beta/src"));
        ensure_dir(&root.join("crates/01-alpha/src"));
        ensure_dir(&root.join("crates/not-a-crate"));
        fs::write(root.join("crates/01-alpha/src/lib.rs"), "").expect("lib.rs should be created");
        fs::write(root.join("crates/02-beta/src/lib.rs"), "").expect("lib.rs should be created");

        let challenges = discover_challenges(&root).expect("challenge discovery should work");
        let ids: Vec<&str> = challenges.iter().map(|entry| entry.id.as_str()).collect();

        assert_eq!(ids, vec!["01-alpha", "02-beta"]);

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }

    #[test]
    fn ensure_local_workspace_seeds_from_sibling_challenges_workspace() {
        let root = temp_dir("seed");
        let repo = root.join("from-zero-to-systems");
        let challenges = root.join("from-zero-to-systems-challenges");
        ensure_dir(&repo);
        ensure_dir(&challenges.join("crates/01-alpha/src"));
        ensure_dir(&challenges.join("play/src"));
        fs::write(
            challenges.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/01-alpha\", \"play\"]\n",
        )
        .expect("Cargo.toml should be created");
        fs::write(challenges.join("Cargo.lock"), "").expect("Cargo.lock should be created");
        fs::write(
            challenges.join("crates/01-alpha/Cargo.toml"),
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("crate manifest should be created");
        fs::write(challenges.join("crates/01-alpha/src/lib.rs"), "// stub\n")
            .expect("stub should be created");
        fs::write(
            challenges.join("play/Cargo.toml"),
            "[package]\nname = \"play\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("play manifest should be created");
        fs::write(challenges.join("play/src/main.rs"), "fn main() {}\n")
            .expect("play main should be created");

        let local = ensure_local_workspace(&repo).expect("local workspace should be created");

        assert_eq!(local, repo.join(".fzts/workspace"));
        assert_eq!(
            fs::read_to_string(local.join("crates/01-alpha/src/lib.rs"))
                .expect("seeded stub should exist"),
            "// stub\n"
        );
        assert!(
            local.join("play/src/main.rs").exists(),
            "play crate should be copied"
        );

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }

    #[test]
    fn ensure_local_workspace_errors_without_seed_source() {
        let root = temp_dir("seed-missing");
        let repo = root.join("from-zero-to-systems");
        ensure_dir(&repo);

        let error = ensure_local_workspace(&repo).expect_err("missing seed source should error");

        assert!(error
            .to_string()
            .contains("from-zero-to-systems-challenges"));

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }
}
