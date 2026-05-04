export { Card } from "./Card";
export type { CardProps } from "./Card";
export { Modal } from "./Modal";
export type { ModalProps } from "./Modal";

export const surfacePrimitiveNames = ["Card", "Modal", "Panel"] as const;

export type SurfacePrimitiveName = (typeof surfacePrimitiveNames)[number];
