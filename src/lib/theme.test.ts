import { describe, expect, it } from "vitest";
import { nextTheme } from "./theme";

describe("theme transitions", () => {
  it("toggles between the two explicit themes", () => {
    expect(nextTheme("light")).toBe("dark");
    expect(nextTheme("dark")).toBe("light");
  });
});
