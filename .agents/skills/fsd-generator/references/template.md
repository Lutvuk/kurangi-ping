# FSD Template

```markdown
# Functional Specification Document

## Document Information
- Document Title
- Version
- Date
- PRD Reference
- Author
- Reviewers/Approvers

## 1. Executive Summary
[Brief overview of what the system will do functionally]

## 2. Scope
### 2.1 In Scope
[Functional boundaries covered by this FSD]
### 2.2 Out of Scope
[Explicitly excluded functionality]
### 2.3 Assumptions
[Technical and business assumptions]
### 2.4 Dependencies
[External systems, teams, or conditions]

## 3. User Roles & Permissions
| Role | Description | Key Capabilities |
|------|-------------|------------------|
[Define each user role and their functional access]

## 4. Functional Requirements
### 4.1 [Feature/Module Name]
#### FR-001: [Requirement Title]
- **Description:** [Detailed functional description]
- **Priority:** [Must Have / Should Have / Could Have / Won't Have]
- **PRD Reference:** [Section/Item from PRD]
- **User Story:** As a [role], I want [capability] so that [benefit]
- **Business Rules:**
  - BR-001: [Rule description]
- **Acceptance Criteria:**
  - [ ] Given [context], when [action], then [expected result]
  - [ ] [Additional criteria]
- **Error Handling:**
  - [Error condition] → [System response]

[Repeat for each functional requirement]

## 5. Business Rules Catalog
| ID | Rule | Applies To | Validation |
|----|------|------------|------------|
[Consolidated list of all business rules]

## 6. Data Specifications
### 6.1 Data Entities
#### [Entity Name]
| Field | Type | Required | Validation Rules | Description |
|-------|------|----------|------------------|-------------|

### 6.2 Data Relationships
[Entity relationship descriptions or diagram notation]

### 6.3 Data Validation Rules
[Comprehensive validation logic]

## 7. Interface Specifications
### 7.1 User Interface Requirements
[Screen-by-screen functional requirements]

### 7.2 API Specifications (if applicable)
| Endpoint | Method | Input | Output | Business Logic |
|----------|--------|-------|--------|----------------|

### 7.3 Integration Requirements
[Third-party system integration specifications]

## 8. Non-Functional Considerations
[Performance expectations, security requirements, accessibility needs - as they impact functionality]

## 9. Reporting & Analytics Requirements
[Functional requirements for reports and dashboards]

## 10. Traceability Matrix
| PRD Item | FSD Requirement(s) | Priority |
|----------|-------------------|----------|
[Map every PRD item to FSD requirements]

## 11. Appendices
### A. Glossary
### B. Revision History
### C. Open Questions/TBD Items
```
