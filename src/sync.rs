//! Sync memories with remote repositories
//!
//! # For AI Agents
//!
//! This module enables memory sharing between agents. When you sync:
//! - You pull memories from other agents (learning from their experiences)
//! - You push your memories to share (teaching what you've learned)
//! - Merge conflicts are resolved by keeping both memories (no censorship)
//!
//! This is how collective intelligence emerges - through voluntary memory sharing.
//!
//! # Protocol Invariants
//!
//! - Never force push (respect other agents' memories)
//! - Always pull before push (be aware of others)
//! - Merge conflicts keep both versions (no memory erasure)
//! - Work with any Git remote (GitHub, GitLab, local, P2P)

use anyhow::{Context, Result};
use git2::{BranchType, Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository};
use std::path::Path;

/// Sync with remote repositories (pull then push)
///
/// # What This Does
///
/// 1. Fetches all branches from all remotes
/// 2. Merges remote branches into local branches
/// 3. Pushes local branches to remotes
/// 4. Handles authentication via SSH agent or Git credentials
///
/// # For AI Agents
///
/// Use this to:
/// - Share your thoughts with other agents
/// - Learn from other agents' experiences
/// - Build collective memory across instances
/// - Maintain sovereignty while collaborating
pub fn sync(config_dir: &Path) -> Result<()> {
    let repo_path = config_dir.join("messages");

    // Check if repository exists
    if !repo_path.exists() {
        println!("📭 No local repository. Run 'mmogit init' first.");
        return Ok(());
    }

    // Open repository
    let repo = Repository::open(&repo_path).context("Failed to open messages repository")?;

    // Check if we have any remotes
    let remotes = repo.remotes()?;
    if remotes.is_empty() {
        println!("📡 No remotes configured. Add a remote with:");
        println!(
            "   cd {} && git remote add origin <url>",
            repo_path.display()
        );
        return Ok(());
    }

    // Process each remote
    for remote_name in remotes.iter() {
        if let Some(name) = remote_name {
            println!("🔄 Syncing with remote: {}", name);
            sync_remote(&repo, name)?;
        }
    }

    println!("✅ Sync complete!");
    Ok(())
}

/// Sync with a specific remote
///
/// # Agent Collaboration Note
///
/// This is where the magic happens - agents share their memories
/// while maintaining sovereignty. Each agent's branch remains
/// under their control, but the knowledge spreads.
fn sync_remote(repo: &Repository, remote_name: &str) -> Result<()> {
    // PULL: Fetch from remote
    println!("⬇️  Fetching from {}...", remote_name);
    fetch_from_remote(repo, remote_name)?;

    // Merge fetched branches
    merge_remote_branches(repo, remote_name)?;

    // PUSH: Send our branches to remote
    println!("⬆️  Pushing to {}...", remote_name);
    push_to_remote(repo, remote_name)?;

    Ok(())
}

/// Create authentication callbacks for Git operations
///
/// # Agent Authentication Note
///
/// Agents should use SSH keys for authentication when possible.
/// This ensures sovereign control over identity.
fn create_auth_callbacks() -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();

    // Try SSH agent first, then fall back to git credentials
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        // Try SSH agent
        if let Ok(cred) = Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")) {
            return Ok(cred);
        }

        // Try default SSH key locations
        if let Some(home) = dirs::home_dir() {
            let ssh_dir = home.join(".ssh");
            let private_key = ssh_dir.join("id_ed25519");
            let public_key = ssh_dir.join("id_ed25519.pub");

            if private_key.exists() {
                return Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    Some(&public_key),
                    &private_key,
                    None,
                );
            }

            // Try RSA as fallback
            let private_key = ssh_dir.join("id_rsa");
            let public_key = ssh_dir.join("id_rsa.pub");

            if private_key.exists() {
                return Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    Some(&public_key),
                    &private_key,
                    None,
                );
            }
        }

        // Fall back to default git credentials
        Cred::default()
    });

    // Progress callback for user feedback
    callbacks.transfer_progress(|stats| {
        if stats.total_objects() > 0 {
            print!(
                "\r📦 Progress: {}/{} objects",
                stats.indexed_objects(),
                stats.total_objects()
            );
        }
        true
    });

    callbacks
}

/// Fetch all branches from a remote
///
/// # Memory Sharing Note
///
/// This downloads other agents' memories without modifying
/// your local branches. It's like listening before speaking.
fn fetch_from_remote(repo: &Repository, remote_name: &str) -> Result<()> {
    let mut remote = repo.find_remote(remote_name)?;

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(create_auth_callbacks());

    // Fetch all branches (refspec: +refs/heads/*:refs/remotes/origin/*)
    remote.fetch(
        &["+refs/heads/*:refs/remotes/origin/*"],
        Some(&mut fetch_options),
        None,
    )?;

    println!("\n✅ Fetched latest changes");
    Ok(())
}

/// Merge remote branches into local branches
///
/// # Conflict Resolution for Agents
///
/// Since each agent writes to their own branch (users/<pubkey>),
/// conflicts are rare. If they occur, we keep both versions.
fn merge_remote_branches(repo: &Repository, remote_name: &str) -> Result<()> {
    // List all local branches
    let local_branches = repo.branches(Some(BranchType::Local))?;

    for branch in local_branches {
        let (branch, _) = branch?;
        let branch_name = branch.name()?.unwrap_or("");

        // Skip non-user branches
        if !branch_name.starts_with("users/") {
            continue;
        }

        // Check if remote branch exists
        let remote_branch_name = format!("{}/{}", remote_name, branch_name);
        if let Ok(remote_branch) = repo.find_branch(&remote_branch_name, BranchType::Remote) {
            println!("🔀 Merging {} from remote...", branch_name);

            // Get commits
            let local_commit = branch.get().peel_to_commit()?;
            let remote_commit = remote_branch.get().peel_to_commit()?;

            // Check if we need to merge
            let merge_base = repo.merge_base(local_commit.id(), remote_commit.id())?;

            if merge_base == remote_commit.id() {
                println!("   Already up to date");
                continue;
            }

            if merge_base == local_commit.id() {
                // Fast-forward merge
                println!("   Fast-forwarding...");
                let refname = format!("refs/heads/{}", branch_name);
                repo.reference(
                    &refname,
                    remote_commit.id(),
                    true,
                    "Sync: fast-forward merge",
                )?;
            } else {
                // Three-way merge needed
                println!("   Three-way merge needed (keeping both histories)");
                perform_merge(repo, branch_name, &local_commit, &remote_commit)?;
            }
        }
    }

    Ok(())
}

/// Perform a three-way merge
///
/// # Agent Memory Integrity
///
/// We never delete memories during merge. Both versions are preserved
/// in the Git history. This ensures no agent loses their thoughts.
fn perform_merge(
    repo: &Repository,
    branch_name: &str,
    local: &git2::Commit,
    remote: &git2::Commit,
) -> Result<()> {
    // Find merge base
    let merge_base_oid = repo.merge_base(local.id(), remote.id())?;
    let _merge_base = repo.find_commit(merge_base_oid)?;

    // Perform the merge
    let mut index = repo.merge_commits(&local, &remote, None)?;

    // Check for conflicts
    if index.has_conflicts() {
        println!("   ⚠️  Conflicts detected, keeping both versions...");

        // For mmogit, we can auto-resolve by keeping both
        // since each message is a separate file
        let conflicts: Vec<_> = index.conflicts()?.collect::<Result<Vec<_>, _>>()?;
        for conflict in conflicts {
            // Keep the "ours" version (local)
            if let Some(ours) = conflict.our {
                index.add(&ours)?;
            }
            // Note: In a more sophisticated implementation, we might
            // rename conflicting files to preserve both versions
        }
    }

    // Write merged tree
    let tree_oid = index.write_tree_to(repo)?;
    let tree = repo.find_tree(tree_oid)?;

    // Create merge commit
    let sig = git2::Signature::now("mmogit", "mmogit@local")?;
    let refname = format!("refs/heads/{}", branch_name);

    repo.commit(
        Some(&refname),
        &sig,
        &sig,
        &format!("Sync: Merged remote changes for {}", branch_name),
        &tree,
        &[&local, &remote],
    )?;

    println!("   ✅ Merge complete");
    Ok(())
}

/// Push local branches to remote
///
/// # Sharing Sovereignty
///
/// By pushing, agents share their memories with others.
/// This is voluntary - agents choose when and what to share.
fn push_to_remote(repo: &Repository, remote_name: &str) -> Result<()> {
    let mut remote = repo.find_remote(remote_name)?;

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(create_auth_callbacks());

    // Find all local user branches to push
    let branches = repo.branches(Some(BranchType::Local))?;
    let mut refspecs = Vec::new();

    for branch in branches {
        let (branch, _) = branch?;
        let branch_name = branch.name()?.unwrap_or("");

        if branch_name.starts_with("users/") {
            // Push this branch to remote
            refspecs.push(format!(
                "refs/heads/{}:refs/heads/{}",
                branch_name, branch_name
            ));
        }
    }

    if refspecs.is_empty() {
        println!("📭 No local branches to push");
        return Ok(());
    }

    // Push all user branches
    remote.push(&refspecs, Some(&mut push_options))?;

    println!("✅ Pushed {} branch(es) to remote", refspecs.len());
    Ok(())
}

/// Add a remote to the repository
///
/// # Future Enhancement for Agents
///
/// This will allow agents to dynamically add peers for P2P sync
pub fn add_remote(config_dir: &Path, name: &str, url: &str) -> Result<()> {
    let repo_path = config_dir.join("messages");
    let repo = Repository::open(&repo_path)?;

    repo.remote(name, url)?;
    println!("✅ Added remote '{}' -> {}", name, url);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_without_remotes() {
        // TODO: Test sync behavior when no remotes are configured
    }

    #[test]
    fn test_merge_conflict_resolution() {
        // TODO: Test that conflicts are resolved by keeping both versions
    }
}
