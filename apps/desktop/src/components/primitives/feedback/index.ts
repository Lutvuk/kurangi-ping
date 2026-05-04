export const feedbackPrimitiveNames = ["StatusBadge", "Toast"] as const;

export type FeedbackPrimitiveName = (typeof feedbackPrimitiveNames)[number];
