export { StatusBadge } from "./StatusBadge";
export type { StatusBadgeProps } from "./StatusBadge";
export { Toast } from "./Toast";
export type { ToastProps, ToastTone } from "./Toast";

export const feedbackPrimitiveNames = ["StatusBadge", "Toast"] as const;

export type FeedbackPrimitiveName = (typeof feedbackPrimitiveNames)[number];
