---
name: handle_dependabot_prs
description: "Strict workflow for fixing and merging failing Dependabot pull requests."
---

# Handling Dependabot PRs

When tasked with fixing failing Dependabot pull requests (especially major version bumps with breaking changes), you MUST follow this strict, systematic workflow. 

## Strict Production Rules
- **Mandatory CI Pipelines:** NEVER merge a Dependabot PR if any test, lint, or security scan fails.
- **DO NOT** push directly to `main`. Always push fixes to the specific `dependabot/` branch.
- **DO NOT** manually edit `Cargo.lock`, `package-lock.json`, or `poetry.lock` without running the native build/update command (e.g., `cargo build`, `npm update`).
- **Required Human Approval:** Do not "rubber-stamp" automated PRs. Ensure code changes resulting from API bumps are vetted.
- **Separation of Concerns:** Differentiate between `development` and `production` dependencies. Major version bumps in production dependencies require extensive local verification.

## Phase 1: Diagnose and Triage
1. **Identify the failing PRs:** Determine which dependency is failing and whether it's a dev or prod dependency.
2. **Review GitHub Actions Logs:** Open the PR and identify the exact step that failed (Compilation, Linting, or Testing).
3. **Categorize the Bump:** For major version bumps, assume breaking API changes. *ALWAYS read the migration guide or CHANGELOG.md before changing code.*

## Phase 2: Local Verification & Fixing
For each failing Dependabot PR, repeat this process in isolation:
1. **Checkout the branch:** `git checkout <dependabot-branch-name>`
2. **Build locally:** `cargo build --workspace` (or equivalent for npm/python).
3. **Analyze & Research:** Locate compiler errors, cross-reference with the migration guide, and identify the required syntax/API changes.
4. **Fix:** Adjust the codebase to align with the new API. Do not use generic hacks; fix the root cause.
5. **Test:** Run tests locally (`cargo test --workspace`). Update assertions or mocks if the underlying behavior legitimately changed.
6. **Commit & Push to Branch:** 
   - `git add .`
   - `git commit -m "fix: adapt to breaking API changes in <dependency> v<version>"`
   - `git push origin <dependabot-branch-name>` (Never force push unless rewriting history on the PR branch).

## Phase 3: Pipeline Validation
1. Monitor the CI pipeline on the updated Dependabot branch.
2. **Strict Gate:** Wait for the CI to turn green. If it fails again, repeat Phase 2.

## Phase 4: Clean Merge
1. Once CI passes, perform a final review of the diff.
2. Merge the PR using **"Squash and merge"** to keep the `main` history clean.
3. Delete the Dependabot branch.

## Phase 5: Post-Merge Integration Testing
1. `git checkout main && git pull --rebase`
2. Run a full local build and test suite (`cargo build --release --workspace && cargo test --release --workspace`).
3. If integration issues emerge between multiple updated dependencies, fix them immediately in a new PR.

## Best Practice Recommendations for repository owners
When advising users on repository configuration, recommend:
- Extending update intervals to weekly or monthly to batch changes.
- Using `dependabot.yml` `groups` to bundle related dependencies (e.g., all linting tools).
- Implementing a 30-day cooldown for new major versions to avoid adopting unstable releases.
