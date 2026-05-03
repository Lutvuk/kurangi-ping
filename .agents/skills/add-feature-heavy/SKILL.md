---
description: HEAVY complexity feature addition — generates documentation suite
  consistent with the project kickoff heavy workflow.
name: Add Feature Heavy
---

# Add Feature Workflow (HEAVY)

> [!IMPORTANT]
> **Rule Zero (Anti-Bias):** Do NOT skip steps or assume completion based on filenames. You must verify every step explicitly with `notify_user` before generating files.
> **Blocking Assertion:** You are NOT allowed to use `write_to_file` until you have called `notify_user` with the specific list of sections/diffs you are about to write.
> **Validation (Intent-Based):** You must wait for a clear affirmative **INTENT** (e.g., "ok", "yes", "looks good", "proceed"). Strictly reject ambiguous replies (e.g., "maybe later", "not sure").
> **Invariant:** Skills are READ-ONLY. They return content; they do NOT write files. The Workflow performs the write after approval.

This workflow is for adding new features or modules to an **enterprise, platform, regulated, or multi-team project**. It ensures consistent documentation standards for everything added post-kickoff.

## Step 1: Feature-Specific Requirements & Delta (Addendum Mode)

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

## Step 2: Feature Story Decomposition (L1-L5)

1.  **Read**: Load `skills/story-generator/SKILL.md`.
2.  **Verify**: Call `notify_user` with the list of stories (titles only) grouped by layer.
3.  **Wait**: Wait for "APPROVED: stories" or revision requests.
4.  **Revision Loop**:
    *   If changes requested: Update draft, re-call `notify_user` (Preview), and Wait again.
    *   Repeat until APPROVED.
5.  **Write**: Execute `write_to_file` to create story files at `docs/features/[feature-name]/stories/`.

## Step 3: Global Artifacts Integration

> [!IMPORTANT]
> **Merge Safety:** You must compute and show the EXACT diffs before modifying any global artifact.

1.  **Objective**: Merge feature-specific artifacts back into the core.
2.  **Read**: Load `skills/erd-generator/SKILL.md` and `skills/api-contract-generator/SKILL.md` (if applicable).
3.  **Plan Merge**:
    *   *ERD*: Draft the new Mermaid entities for `docs/erd/core-erd.md`.
    *   *API*: Draft the new endpoints for `docs/api/contracts/`.
    *   *PRD*: Draft the new section for `docs/prd.md`.
3.  **Targeted Section Preview**: Call `notify_user` with the **specific section content** you are changing.
    *   *Do NOT* dump the entire file.
    *   *Example*: "I am replacing the `## 3.1 User Schema` block with the following..."
4.  **Wait**: Wait for affirmative intent regarding the merge.
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

*   Notify the user that feature documentation is complete.
*   Provide a summary of files created and their locations.
*   Remind user that the **Global Dependency Graph** in `docs/dependency-graph.md` shows the recommended build order across all features.
*   Remind user that ERD and API changes are explicit, audited, and merged back into global artifacts.
