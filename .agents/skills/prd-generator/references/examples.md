# PRD Examples & Formats

## User Story Format

| ID | User Story | Acceptance Criteria | Design | Notes | Platform | JIRA Ticket |
|----|------------|---------------------|--------|-------|----------|-------------|
| US-01 | As a returning user, I want to see my purchase history so that I can reorder items quickly | **Given** I'm logged into my account<br>**When** I navigate to "My Orders"<br>**Then** I see my last 10 orders sorted by date<br>**And** each order shows items, date, and total<br>**And** I can click "Reorder" on any item | [Figma link] | Cache for performance | iOS/Android/Web | PROJ-123 |

## Analytics Event Format

```json
{
  "Trigger": "Click",
  "TriggerValue": "Checkout Button",
  "Page": "Shopping Cart",
  "Data": {
    "CartValue": 149.99,
    "ItemCount": 3,
    "UserSegment": "Premium"
  },
  "Description": "User initiates checkout from cart page"
}
```

## Example Input

> "Create a PRD for [Feature Name]: [Brief description]. This will solve [Problem] for [Target Users]. Key features include [Feature 1], [Feature 2], [Feature 3]. Success will be measured by [Metric]. We need this by [Timeline]."
