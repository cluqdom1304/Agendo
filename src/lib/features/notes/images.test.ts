// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { installLayoutStubs } from "../../testing";
import { ImageCache, imageFiles } from "./imageCache";
import NoteEditor from "./NoteEditor.svelte";
import type { ImageHost } from "./attachmentImage";

installLayoutStubs();

const file = (name: string, type: string, bytes = [1, 2, 3]) =>
  new File([new Uint8Array(bytes)], name, { type });

describe("imageFiles", () => {
  it("keeps only the types the store accepts", () => {
    const files = [
      file("a.png", "image/png"),
      file("b.svg", "image/svg+xml"),
      file("c.txt", "text/plain"),
      file("d.webp", "image/webp"),
    ];
    expect(imageFiles(files).map((f) => f.name)).toEqual(["a.png", "d.webp"]);
    expect(imageFiles(null)).toEqual([]);
  });
});

describe("ImageCache", () => {
  function make(read = async () => new Uint8Array([9])) {
    const created: Blob[] = [];
    const revoked: string[] = [];
    const show = vi.fn();
    const failed = vi.fn();
    const add = vi.fn(async () => ({
      id: "a".repeat(64),
      mime: "image/png",
      sizeBytes: 3,
    }));
    const cache = new ImageCache({
      api: { add, read },
      show,
      failed,
      urls: {
        create: (blob) => {
          created.push(blob);
          return `blob:${created.length}`;
        },
        revoke: (url) => void revoked.push(url),
      },
    });
    return { cache, created, revoked, show, failed, add };
  }

  it("reads each image once and gives it a typed address", async () => {
    const read = vi.fn(async () => new Uint8Array([9]));
    const { cache, created } = make(read);
    expect(await cache.url("x", "image/png")).toBe("blob:1");
    expect(await cache.url("x", "image/png")).toBe("blob:1");
    expect(read).toHaveBeenCalledTimes(1);
    expect(created[0]?.type).toBe("image/png");
  });

  it("answers null for an image whose file is gone", async () => {
    const { cache } = make(async () => {
      throw "NOT_FOUND: no está";
    });
    expect(await cache.url("x", "image/png")).toBeNull();
  });

  it("stores files, reports a refusal and previews", async () => {
    const { cache, add, failed, show } = make();
    expect(await cache.add(file("a.png", "image/png"))).toEqual({
      id: "a".repeat(64),
      mime: "image/png",
    });
    expect(add).toHaveBeenCalledWith(new Uint8Array([1, 2, 3]));

    add.mockRejectedValueOnce("VALIDATION: no es una imagen");
    expect(await cache.add(file("b.png", "image/png"))).toBeNull();
    expect(failed).toHaveBeenCalledWith("VALIDATION: no es una imagen");

    cache.preview("x", "image/png");
    await waitFor(() =>
      expect(show).toHaveBeenCalledWith("x", "image/png", "blob:1"),
    );
  });

  it("revokes every address on dispose", async () => {
    const { cache, revoked } = make();
    await cache.url("x", "image/png");
    await cache.url("y", "image/png");
    cache.dispose();
    await waitFor(() => expect(revoked.sort()).toEqual(["blob:1", "blob:2"]));
  });
});

describe("images in the editor", () => {
  const id = "b".repeat(64);
  const doc = JSON.stringify({
    type: "doc",
    content: [
      { type: "paragraph", content: [{ type: "text", text: "antes" }] },
      { type: "image", attrs: { attachmentId: id, mime: "image/png" } },
    ],
  });

  function host(overrides: Partial<ImageHost> = {}): ImageHost {
    return {
      url: vi.fn(async () => "blob:ok"),
      add: vi.fn(async () => ({ id, mime: "image/png" })),
      preview: vi.fn(),
      ...overrides,
    };
  }

  async function mount(images: ImageHost, content = doc) {
    const save = vi.fn<(json: string) => Promise<void>>(async () => {});
    const view = render(NoteEditor, {
      contentJson: content,
      save,
      // A picture as the last block makes the first focus a node selection, which the view
      // finishes in a timer that may outlive the test.
      autofocus: false,
      images,
    });
    const box = await screen.findByRole("textbox");
    return { box, save, view };
  }

  it("draws a stored image from its address and previews it on click", async () => {
    const images = host();
    const { box } = await mount(images);
    const img = await waitFor(() => {
      const found = box.querySelector<HTMLImageElement>("figure img");
      if (!found || !found.getAttribute("src")) throw new Error("not yet");
      return found;
    });
    expect(img.getAttribute("src")).toBe("blob:ok");
    expect(images.url).toHaveBeenCalledWith(id, "image/png");
    await fireEvent.click(img);
    expect(images.preview).toHaveBeenCalledWith(id, "image/png");
  });

  it("says so when the file of an image is gone", async () => {
    const { box } = await mount(host({ url: vi.fn(async () => null) }));
    await waitFor(() =>
      expect(box.querySelector("figure.is-gone")?.textContent).toContain(
        "Imagen no disponible",
      ),
    );
  });

  it("stores a pasted image and inserts it, ignoring other files", async () => {
    const images = host();
    const { box, save } = await mount(images, '{"type":"doc","content":[]}');
    const paste = new Event("paste", { bubbles: true, cancelable: true });
    Object.defineProperty(paste, "clipboardData", {
      value: {
        files: [file("p.png", "image/png"), file("n.txt", "text/plain")],
        getData: () => "",
        types: ["Files"],
      },
    });
    box.dispatchEvent(paste);
    expect(paste.defaultPrevented).toBe(true);
    await waitFor(() => expect(box.querySelectorAll("figure")).toHaveLength(1));
    expect(images.add).toHaveBeenCalledTimes(1);
    await waitFor(
      () =>
        expect(save.mock.calls.at(-1)?.[0] as string).toContain(
          `"attachmentId":"${id}"`,
        ),
      { timeout: 3000 },
    );
  });

  it("leaves a paste without images to the editor", async () => {
    const images = host();
    const { box } = await mount(images);
    const paste = new Event("paste", { bubbles: true, cancelable: true });
    Object.defineProperty(paste, "clipboardData", {
      value: { files: [], getData: () => "" },
    });
    box.dispatchEvent(paste);
    expect(images.add).not.toHaveBeenCalled();
  });

  it("resizes with the keyboard and keeps the width in the document", async () => {
    const { box, save } = await mount(host());
    const grip = await waitFor(() => {
      const found = box.querySelector<HTMLElement>(".image-grip");
      if (!found) throw new Error("not yet");
      return found;
    });
    await fireEvent.keyDown(grip, { key: "ArrowRight" });
    await waitFor(
      () => expect(save.mock.calls.at(-1)?.[0] as string).toContain('"width"'),
      { timeout: 3000 },
    );
  });

  it("does not know images when the host is not given", async () => {
    render(NoteEditor, {
      contentJson: doc,
      save: async () => {},
      autofocus: true,
    });
    const box = await screen.findByRole("textbox");
    expect(box.querySelector("figure")).toBeNull();
  });
});
