---
description: Start working on a story — automatically loads all relevant context
  documents (Design System, Wireframes, TDD, Stories, API Contract, etc.) to
  begin implementation.
name: Start Story
---

# Start Story Workflow

Use this workflow when you're ready to begin implementing a specific story. It automatically loads all necessary context documents so you don't have to manually reference each one.

---

## Step 1: Identify the Story

1.  **Ask**: "Which story are you working on? (Provide ID or Title)"
2.  **Analyze**: Determine **Domain Focus** (Frontend, Backend, or Full Stack) by:
    *   Scanning the selected story content for keywords (e.g., "API", "Database", "UI", "Component").
    *   Checking the currently open files or file structure.
    *   *Default*: If uncertain, assume **Full Stack**.
3.  **Locate**: Find the story file in `docs/features/[feature-name]/stories/` (Heavy) or check `docs/todo.md` (Light).
4.  **Confirm**: Display the story details and the **inferred** project mode & domain focus.
    *   *Ask*: "Is the inferred context (e.g., Heavy/Backend) correct?"
5.  **Approve**: Wait for user approval before proceeding.

---

## Step 2: Load Context Documents

Automatically read and understand the documents based on **Project Mode** and **Domain Focus**.

### A. Check Project Mode first

1.  **Check**: Does `docs/spec.md` exist?
    *   **YES (Light Mode)**: 
        *   Read `docs/spec.md` (Integrated Spec).
        *   Read `docs/todo.md` (Task List).
        *   *Stop loading other documents.*
    *   **NO (Heavy Mode)**: Proceed to section B.

### B. Heavy Mode Loading (Conditional)

Load documents based on the **inferred or confirmed** Domain Focus:

#### 1. Always Load (Core Context)
*   **Stories**: Read all files in `docs/features/[feature-name]/stories/`.
*   **FSD**: Read `docs/features/[feature-name]/fsd.md` (if exists).
*   **PRD Addendum**: Read `docs/features/[feature-name]/prd-addendum.md` (if exists).

#### 2. If FRONTEND or FULL STACK
*   **Design System**: Read `docs/design-system.md`.
*   **Wireframes**: Read `docs/features/[feature-name]/wireframes.md`.
*   **API Contract**: Read `docs/features/[feature-name]/api-contract.md` (for integration).

#### 3. If BACKEND or FULL STACK
*   **Core ERD**: Read `docs/erd/core-erd.md`.
*   **ERD Extension**: Read `docs/features/[feature-name]/erd-extension.md`.
*   **API Standards**: Read `docs/api-standards.md`.
*   **TDD**: Read `docs/features/[feature-name]/tdd.md`.

---

## Step 3: Summarize Context

Provide a brief summary to the user:

1.  **Story Overview**: What the story aims to achieve.
2.  **Relevant Wireframe(s)**: Which screens are affected.
3.  **Technical Approach**: Key implementation points from the TDD.
4.  **API Endpoints**: Any endpoints to implement or consume.
5.  **Dependencies**: Related stories or components.

---

## Step 4: Begin Implementation

1.  **Ask**: "Ready to start? Any questions before we begin coding?"
2.  **Clarify**: Address any questions or ambiguities.
3.  **Implement**: Start coding with full context loaded.
4.  **Error Handling Protocol (CRITICAL)**:
    *   If you encounter an error and fail to fix it **3 times**:
        *   **STOP** immediately. Do not attempt a 4th blind fix.
        *   **Escalate** to the user.
        *   **Report**: Provide the full error log, the attempted fixes, and relevant code.
        *   **Ask**: Request specific guidance, reference code, or a manual fix from the user.

---

# Notes

-   This workflow assumes the project has gone through the kickoff workflow and has the standard documentation structure.
-   If any document is missing, notify the user and proceed with available context.
-   Keep all loaded context in mind throughout the implementation session.
