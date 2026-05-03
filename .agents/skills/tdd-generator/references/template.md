# TDD Output Template

Use this template structure for the output. Omit any section entirely if no delta exists.

```markdown
# Technical [Design Document | Spec]
**Mode:** [LIGHT | HEAVY]
**Project:** [Extracted from FSD]
**Version:** 1.0
**Date:** [Current Date]
**Status:** Draft

---

## 1. Executive Summary
- Brief overview of the NEW functionality (2-3 paragraphs).
- Key technical decisions summary.
- Technology stack overview (if new).

## 2. System Architecture

### 2.1 Architecture Overview
- High-level architecture diagram (Mermaid preferred).
- Key architectural principles applied.

### 2.2 Component Architecture
| Component | Responsibility (Delta) | Technology | Dependencies |
|-----------|------------------------|------------|--------------|
| [Name] | [Description] | [Tech] | [Dependencies] |

## 3. Data Architecture (Delta Focus)

### 3.1 Data Model (New/Modified Only)
| Entity | Attribute | Type | Constraints | Description |
|--------|-----------|------|-------------|-------------|
| [Entity] | [Attr] | [Type] | [Constraints] | [Desc] |

### 3.2 Database Design
- Schema design decisions (Delta).
- Indexing strategy (New indexes).

### 3.3 Data Flow
- Data lifecycle for new features (Mermaid sequenceDiagram preferred).

## 4. API Design (Delta Focus)

### 4.1 Endpoint Specifications (New/Modified Only)
For each new/modified endpoint:

**[HTTP Method] [Endpoint Path]**
- **Purpose:** [Description]
- **Authentication:** [Required/Optional, Type]
- **Request:** [Schema or reference to API Contract]
- **Response:** [Schema or reference to API Contract]
- **Error Codes:** [Relevant codes]
- **Business Rules:** [Validation logic]

> Reference the API Contract for full schemas. Only document deviations or extensions here.

### 4.2 Authentication & Authorization (New/Modified Only)
- Only document changes to auth flow.

## 5. Component Design (Delta Focus)

### 5.1 Backend Services (New/Modified Only)
**[Service Name]**
- **Responsibility:** [Description]
- **Interfaces:** [Input/Output]
- **Key Classes/Functions:**
  - [Class/Function]: [Purpose]

### 5.2 Frontend Architecture (New/Modified Only)
Include ONLY if the feature introduces new UI behavior or state management.

| Wireframe Screen | Component(s) | State Requirements | API Calls |
|------------------|--------------|-------------------|-----------|
| [Screen] | [Components] | [State] | [APIs] |

### 5.3 Integration Layer (New Only)
- New external system integrations.
- New message queues/events.

## 6. Performance & Scalability (New Considerations Only)
If no new risks, omit this section entirely.

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| [Metric] | [Value] | [How] |

## 7. Failure Modes & Edge Cases (Delta Only)
- Dependency failure handling
- Retry / idempotency behavior
- Partial success scenarios
- Data consistency risks

## 8. Technical Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| [Risk] | High/Med/Low | High/Med/Low | [Strategy] |

## 9. Dependencies & Assumptions
- New third-party services or libraries.
- Technical assumptions made.

## 10. Appendices (as needed)
- Technology Stack (New Only)
- Glossary
- Reference Documents
```
