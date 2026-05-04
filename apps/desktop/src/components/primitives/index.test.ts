import { describe, expect, it } from "vitest";
import {
  actionPrimitiveNames,
  feedbackPrimitiveNames,
  formPrimitiveNames,
  primitiveCatalog,
  surfacePrimitiveNames
} from "./index";

describe("primitive barrel exports", () => {
  it("exposes deterministic category catalogs", () => {
    expect(actionPrimitiveNames).toEqual(["Button", "PrimaryToggle"]);
    expect(formPrimitiveNames).toEqual(["Input", "Select"]);
    expect(feedbackPrimitiveNames).toEqual(["StatusBadge", "Toast"]);
    expect(surfacePrimitiveNames).toEqual(["Card", "Modal", "Panel"]);
  });

  it("exposes single-entry primitive catalog", () => {
    expect(primitiveCatalog.actions).toEqual(actionPrimitiveNames);
    expect(primitiveCatalog.forms).toEqual(formPrimitiveNames);
    expect(primitiveCatalog.feedback).toEqual(feedbackPrimitiveNames);
    expect(primitiveCatalog.surfaces).toEqual(surfacePrimitiveNames);
  });
});
