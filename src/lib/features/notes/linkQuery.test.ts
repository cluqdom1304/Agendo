import { describe, expect, it } from "vitest";

import { findLinkQuery, plainQuery } from "./linkQuery";

describe("findLinkQuery", () => {
  it("finds the text typed after [[", () => {
    expect(findLinkQuery("[[")).toBe("");
    expect(findLinkQuery("ver [[álg")).toBe("álg");
    expect(findLinkQuery("a [[Apuntes de clase")).toBe("Apuntes de clase");
  });

  it("ignores text with no open bracket pair or one already closed", () => {
    expect(findLinkQuery("sin enlace")).toBeNull();
    expect(findLinkQuery("[solo uno")).toBeNull();
    expect(findLinkQuery("[[cerrado]] y más")).toBeNull();
    expect(findLinkQuery("[[a[[b")).toBe("b");
  });
});

describe("plainQuery", () => {
  it("drops what the search box would read as a filter or a quote", () => {
    expect(plainQuery("tag:trabajo informe")).toBe("tag trabajo informe");
    expect(plainQuery('a "b"  c')).toBe("a b c");
    expect(plainQuery("  ")).toBe("");
  });
});
