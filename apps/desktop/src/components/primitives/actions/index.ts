export const actionPrimitiveNames = ["Button", "PrimaryToggle"] as const;

export type ActionPrimitiveName = (typeof actionPrimitiveNames)[number];
