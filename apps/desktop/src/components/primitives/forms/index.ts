export const formPrimitiveNames = ["Input", "Select"] as const;

export type FormPrimitiveName = (typeof formPrimitiveNames)[number];
