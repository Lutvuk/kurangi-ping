export * from "./actions";
export * from "./feedback";
export * from "./forms";
export * from "./surfaces";
export * from "./shared/types";

export const primitiveCatalog = {
  actions: ["Button", "PrimaryToggle"],
  forms: ["Input", "Select"],
  feedback: ["StatusBadge", "Toast"],
  surfaces: ["Card", "Modal", "Panel"]
} as const;

export type PrimitiveCatalog = typeof primitiveCatalog;
