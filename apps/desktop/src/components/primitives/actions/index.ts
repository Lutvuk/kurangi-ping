export { Button } from "./Button";
export type { ButtonProps } from "./Button";

export const actionPrimitiveNames = ["Button", "PrimaryToggle"] as const;

export type ActionPrimitiveName = (typeof actionPrimitiveNames)[number];
