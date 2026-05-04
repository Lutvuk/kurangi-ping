export { StatusBadge } from "./StatusBadge";
export type { StatusBadgeProps } from "./StatusBadge";

export const feedbackPrimitiveNames = ["StatusBadge", "Toast"] as const;

export type FeedbackPrimitiveName = (typeof feedbackPrimitiveNames)[number];
