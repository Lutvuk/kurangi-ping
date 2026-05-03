---
description: HEAVY complexity project kickoff — generates full documentation
  suite for enterprise, platform, regulated, or multi-team projects.
name: Project Kickoff Heavy
---

# Project Kickoff Workflow (HEAVY)

> [!IMPORTANT]
> **Rule Zero (Anti-Bias):** Do NOT skip steps or assume completion based on filenames. You must verify every step explicitly with `notify_user` before generating files.
> **Blocking Assertion:** You are NOT allowed to use `write_to_file` until you have called `notify_user` with the specific list of sections/diffs you are about to write.
> **Validation (Intent-Based):** You must wait for a clear affirmative **INTENT** (e.g., "ok", "yes", "looks good", "proceed"). Strictly reject ambiguous replies (e.g., "maybe later", "not sure").
> **Invariant:** Skills are READ-ONLY. Skills return content; they do NOT write files. The Workflow performs the write after approval.

This workflow is for **enterprise, platform, regulated, or multi-team projects** where documentation is a first-class citizen. The goal is to create a robust foundation ("The 5 Pillars") before coding begins.

## When to Use HEAVY

Use this workflow if ANY of the following apply:
- Team size > 3 developers
- Regulated industry (FinTech, HealthTech)
- Long-term maintenance expected (> 1 year)
- Complex domain logic requires audit trails
- External stakeholders need to approve specs

---

# Phase 1: Global Artifacts (The 5 Pillars)

## Step 1: Product Brief (The "Why")

1.  **Ask**: Call `notify_user`: "Do you want to create a Product Brief now? (Recommended for alignment)".
2.  **Decision**:
    *   If **YES**: Proceed.
    *   If **NO**: Create a placeholder `docs/product-brief.md` saying "Skipped (Regenerate later from PRD)" and **continue to Step 2**.
3.  **Read**: Load `skills/product-brief/SKILL.md`.
4.  **Verify**: Call `notify_user` with the list of required sections from the skill.
5.  **Interview**: Conduct the "Gap Analysis" interview as defined in the skill.
    *   *Goal*: Understand the Vision, Audience, and Success Metrics.
6.  **Plan**: Draft the content structure internally.
7.  **Preview**: Call `notify_user` with the outline of `docs/product-brief.md` you intend to write.
8.  **Wait**: Wait for "APPROVED: product-brief" or revision requests.
9.  **Revision Loop**:
    *   If changes requested: Update draft, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
10. **Write**: Execute `write_to_file` to create `docs/product-brief.md`.

## Step 2: Global PRD (The "What")

1.  **Read**: Load `skills/prd-generator/SKILL.md`.
2.  **Verify**: Call `notify_user` to confirm you are in **Mode A: Global PRD**.
3.  **Interview**: Follow the skill's gathering process (batch questions).
4.  **Plan**: Draft the 12 required sections internally.
5.  **Preview**: Call `notify_user` listing the sections you will include.
6.  **Wait**: Wait for "APPROVED: prd" or revision requests.
7.  **Revision Loop**:
    *   If changes requested: Update draft, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
8.  **Write**: Execute `write_to_file` to create `docs/prd.md`.

## Step 3: Tech Stack Definition (No Code Yet)

1.  **Read**: Load `skills/tech-stack/SKILL.md`.
2.  **Verify**: Call `notify_user` with the proposed stack categories (Frontend, Backend, Infra, etc.).
3.  **Action**: Define the technology choices.
4.  **Preview**: Call `notify_user` with the proposed stack summary.
5.  **Wait**: Wait for "APPROVED: tech-stack" or revision requests.
6.  **Revision Loop**:
    *   If changes requested: Update draft, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
7.  **Write**: Execute `write_to_file` to create `docs/tech-stack.md`.

## Step 4: System Architecture & Global Rules (Global FSD)

1.  **Read**: Load `skills/fsd-generator/SKILL.md`.
2.  **Verify**: Call `notify_user` to confirm you are in **Mode A: Global FSD**.
3.  **Plan**: Draft the Architecture, Global Rules, and Data Dictionary.
4.  **Preview**: Call `notify_user` with the proposed sections.
5.  **Wait**: Wait for "APPROVED: fsd" or revision requests.
6.  **Revision Loop**:
    *   If changes requested: Update draft, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
7.  **Write**: Execute `write_to_file` to create `docs/fsd.md`.


## Step 5: Design System

1.  **Read**: Load `skills/design-system/SKILL.md`.
2.  **Verify**: Call `notify_user` with the list of design tokens/components you plan to define.
3.  **Interview**: Follow the skill's gathering process (Brand, Colors, Typography).
4.  **Plan**: Draft the design system structure.
5.  **Preview**: Call `notify_user` with the proposed content of `docs/design-system.md`.
6.  **Wait**: Wait for "APPROVED: design-system" or revision requests.
7.  **Revision Loop**:
    *   If changes requested: Update draft, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
8.  **Write**: Execute `write_to_file` to create the Design System.

## Step 6: Core ERD (Optional)

1.  **Ask**: Call `notify_user` to ask: "Do you want to create a Core ERD? (Required for complex data apps)".
2.  **Decision**:
    *   If **YES**: Proceed.
    *   If **NO**: Create a placeholder `docs/erd/core-erd.md` saying "Skipped" and **STOP** this step.
3.  **Read**: Load `skills/erd-generator/SKILL.md`.
4.  **Verify**: Call `notify_user` to confirm you are in **Mode A: Core ERD**.
5.  **Plan**: Draft the entities and relationships.
6.  **Preview**: Call `notify_user` with the Mermaid diagram source or summary.
7.  **Wait**: Wait for "APPROVED: erd" or revision requests.
8.  **Revision Loop**:
    *   If changes requested: Update draft, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
9.  **Output**: Execute `write_to_file` to create `docs/erd/core-erd.md`.

## Step 7: Core API Standards (Optional)

1.  **Ask**: Call `notify_user` to ask: "Do you want to define Core API Standards?".
2.  **Decision**:
    *   If **YES**: Proceed.
    *   If **NO**: Create a placeholder `docs/api-standards.md` saying "Skipped" and **STOP** this step.
3.  **Interview**: Define auth, error handling, versioning.
4.  **Plan**: Draft the standards.
5.  **Preview**: Call `notify_user` with the proposed standards.
6.  **Wait**: Wait for "APPROVED: api-standards" or revision requests.
7.  **Revision Loop**:
    *   If changes requested: Update draft, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
8.  **Write**: Execute `write_to_file` to create `docs/api-standards.md`.
9.  **Contracts**:
    *   If Standards were approved, propose generating OpenAPI specs.
    *   **Read**: Load `skills/api-contract-generator/SKILL.md`.
    *   **Preview**: Call `notify_user` with the list of contracts.
    *   **Wait**: Wait for affirmative intent or revision requests.
    *   **Revision Loop**:
        *   If changes requested: Update list, re-preview, wait.
        *   Repeat until approved.
    *   **Write**: Create `docs/api/contracts/`.

---

# Feature Discovery

## Step 8: Create Feature List

1.  **Scan**: Use `read_file` or `list_directory` to inspect the repo root.
2.  **Check Foundation**:
    *   **Scaffold**: Look for project indicators (e.g., `package.json`, `go.mod`, `manage.py`).
    *   **DB**: If `docs/erd/core-erd.md` exists, check for schema/migration/model files.
    *   **UI**: If `docs/design-system.md` exists, check for component libraries/structure/layout.
3.  **Classify**: Mark each as `EXISTS`, `PARTIAL`, or `MISSING`.
4.  **Resolve**:
    *   **Scaffold**:
        *   `EXISTS` -> skip, do not add to feature list.
        *   `PARTIAL` -> add `[FOUNDATION] Project Scaffold` with note "extend only, do not overwrite".
        *   `MISSING` -> add `[FOUNDATION] Project Scaffold`.
    *   **DB** (Conditional: only if `docs/erd/core-erd.md` exists):
        *   `EXISTS` -> skip, do not add to feature list.
        *   `PARTIAL` -> add `[FOUNDATION] Base DB Schema` with note "extend only, do not overwrite".
        *   `MISSING` -> add `[FOUNDATION] Base DB Schema`.
    *   **UI** (Conditional: only if `docs/design-system.md` exists):
        *   `EXISTS` -> skip, do not add to feature list.
        *   `PARTIAL` -> add `[FOUNDATION] Base UI Components` with note "extend only, do not overwrite".
        *   `MISSING` -> add `[FOUNDATION] Base UI Components`.
5.  **Read**: `docs/prd.md`.
6.  **Plan**:
    *   **Foundation**: Use the resolved items from Step 4.
    *   **Features**: Identify all remaining features from the PRD "User Stories" or "Requirements".
7.  **Preview**: Call `notify_user` with the proposed `docs/features/feature-list.md`.
8.  **Wait**: Wait for "APPROVED: feature-list" or revision requests.
9.  **Revision Loop**:
    *   If changes requested: Update draft, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
10. **Write**: Execute `write_to_file`.

---

# Per-Feature Artifacts (Repeat Per Feature)

Iterate through each **unchecked item** in `docs/features/feature-list.md`.

## Start Gate: Feature Approval

> **Validation (Intent-Based):** You must wait for a clear affirmative **INTENT** from the user (e.g., "ok", "yes", "looks good", "proceed"). Strictly reject ambiguous replies (e.g., "maybe later", "not sure").
> **Persistence Strategy:** This workflow relies on the state of `docs/features/feature-list.md`. If the session is interrupted, simply restart the workflow; it will skip already checked `[x]` items and resume at the first unchecked item.

1.  **Select**: Pick the next unchecked feature.
2.  **Ask**: Call `notify_user`: "Ready to start feature '[Feature Name]'?".
3.  **Wait**: Wait for affirmative intent or "SKIP".
    *   If SKIP: Mark as skipped in `feature-list.md` and continue to next item.
    *   If AFFIRMATIVE: Mark as `[x]` (In Progress) and proceed to Step 9.

## Step 9: Feature-Specific Requirements & Delta (Addendum Mode)

1.  **Read**: Load `skills/prd-generator/SKILL.md`.
2.  **Context Refresh**: Explicitly read `docs/tech-stack.md` and `docs/prd.md` to refresh memory.
3.  **Verify**: Call `notify_user` to confirm you are in **Mode B: Feature Addendum**.
4.  **Additional Context**: Read other global artifacts if they exist:
    *   *Conditional*: If `docs/erd/core-erd.md` exists, read it.
    *   *Conditional*: If `docs/api-standards.md` exists, read it.
5.  **Interview**: Follow the skill's gathering process.
6.  **Plan**: Draft the Addendum Requirements.
    *   *Conditional*: Include ERD Delta only if Core ERD exists.
    *   *Conditional*: Include API Contract only if API Standards exist.
7.  **Preview**: Call `notify_user` with the Addendum outline.
8.  **Wait**: Wait for "APPROVED" or revision requests.
9.  **Revision Loop**:
    *   If changes requested: Update draft, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
10. **Write**: Execute `write_to_file` to create `docs/features/[feature-name]/prd-addendum.md`.

## Step 10: Feature Story Decomposition (L1-L5)

1.  **Read**: Load `skills/story-generator/SKILL.md`.
2.  **Verify**: Call `notify_user` with the list of stories (titles only) grouped by layer.
3.  **Wait**: Wait for affirmative intent or revision requests.
4.  **Revision Loop**:
    *   If changes requested: Update draft, re-preview, wait.
    *   Repeat until approved.
5.  **Write**: Execute `write_to_file` to create story files at `docs/features/[feature-name]/stories/`.

## Step 11: Global Artifacts Integration

> [!IMPORTANT]
> **Merge Safety:** You must compute and show the EXACT diffs before modifying any global artifact.

1.  **Objective**: Merge feature-specific artifacts back into the core.
2.  **Plan Merge**:
    *   *ERD*: Draft the new Mermaid entities for `docs/erd/core-erd.md`.
    *   *API*: Draft the new endpoints for `docs/api/contracts/`.
    *   *PRD*: Draft the new section for `docs/prd.md`.
3.  **Targeted Section Preview**: Call `notify_user` with the **specific section content** you are changing.
    *   *Do NOT* dump the entire file.
    *   *Example*: "I am replacing the `## 3.1 User Schema` block with the following..."
4.  **Wait**: Wait for affirmative intent regarding the merge (e.g., "ok", "merge it").
5.  **Revision Loop**:
    *   If changes requested: Recalculate diffs, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
6.  **Execute Merge**:
    *   **Conditional**: If `docs/erd/core-erd.md` exists, update it.
    *   **Conditional**: If `docs/api/contracts/` exists, update it.
    *   **Always**: Update `docs/prd.md`.
7.  **Update Graph**:
    *   Execute `skills/dependency-graph-manager/SKILL.md` to update `docs/dependency-graph/README.md`.
    *   **Preview**: Call `notify_user` with the new graph.
    *   **Wait**: Wait for affirmative intent regarding the graph update.
    *   **Revision Loop**:
        *   If changes requested: Update, re-preview, wait.
    *   **Write**: Save the graph.

---

# Completion

*   Notify the user that all documents have been generated.
*   Provide a summary of files created and their locations.
*   Remind user that the **Global Dependency Graph** in `docs/dependency-graph/README.md` shows the recommended build order across all features.
*   Remind user that ERD and API changes are explicit, audited, and merged back into global artifacts (if applicable).
