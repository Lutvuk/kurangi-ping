export const surfacePrimitiveNames = ["Card", "Modal", "Panel"] as const;

export type SurfacePrimitiveName = (typeof surfacePrimitiveNames)[number];
