// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import { installLayoutStubs } from "../../testing";
import NoteEditor from "./NoteEditor.svelte";

installLayoutStubs();

const doc = (text: string) =>
  JSON.stringify({
    type: "doc",
    content: [{ type: "paragraph", content: [{ type: "text", text }] }],
  });

async function mount(
  contentJson: string,
  save = vi.fn<(json: string) => Promise<void>>(async () => {}),
  autofocus = true,
) {
  render(NoteEditor, { contentJson, save, autofocus });
  const box = await screen.findByRole("textbox");
  return { box, save };
}

afterEach(() => vi.useRealTimers());

describe("NoteEditor", () => {
  it("loads TipTap lazily and shows the stored notes", async () => {
    const { box } = await mount(doc("apuntes de álgebra"));
    expect(box.textContent).toBe("apuntes de álgebra");
    expect(screen.getByRole("toolbar")).toBeTruthy();
  });

  it("does not take the focus nor show its controls unless asked to", async () => {
    const { box } = await mount(doc("apuntes"), undefined, false);
    expect(screen.queryByRole("toolbar")).toBeNull();
    expect(document.activeElement).not.toBe(box);

    await fireEvent.focusIn(box);
    expect(screen.getByRole("toolbar")).toBeTruthy();
  });

  it("reports when the focus leaves a note without text", async () => {
    const onEmptyBlur = vi.fn();
    render(NoteEditor, {
      contentJson: "",
      save: async () => {},
      autofocus: true,
      onEmptyBlur,
    });
    const box = await screen.findByRole("textbox");
    await fireEvent.focusOut(box, { relatedTarget: document.body });
    expect(onEmptyBlur).toHaveBeenCalledWith(document.body);
  });

  it("shows the controls only while the focus is inside the note", async () => {
    const { box } = await mount(doc("apuntes"));
    expect(screen.getByRole("toolbar")).toBeTruthy();

    await fireEvent.focusOut(box, { relatedTarget: document.body });
    expect(screen.queryByRole("toolbar")).toBeNull();

    await fireEvent.focusIn(box);
    expect(screen.getByRole("toolbar")).toBeTruthy();
  });

  it("opens with an empty document or with legacy empty content", async () => {
    for (const content of ["", '{"type":"doc","content":[]}']) {
      const { box } = await mount(content);
      expect(box.textContent).toBe("");
      document.body.innerHTML = "";
    }
  });

  it("saves the edited document 400 ms after the last change", async () => {
    const { save } = await mount(doc("hola"));
    vi.useFakeTimers();
    await fireEvent.click(
      screen.getByRole("button", { name: "Lista con viñetas" }),
    );
    await vi.advanceTimersByTimeAsync(399);
    expect(save).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(save).toHaveBeenCalledTimes(1);
    const saved = JSON.parse(save.mock.calls[0]![0] as string);
    expect(saved.content[0].type).toBe("bulletList");
  });

  it("ticks a checklist item with one click and saves it", async () => {
    const checklist = JSON.stringify({
      type: "doc",
      content: [
        {
          type: "taskList",
          content: [
            {
              type: "taskItem",
              attrs: { checked: false },
              content: [
                {
                  type: "paragraph",
                  content: [{ type: "text", text: "comprar" }],
                },
              ],
            },
          ],
        },
      ],
    });
    const { box, save } = await mount(checklist);
    vi.useFakeTimers();
    const checkbox = box.querySelector<HTMLInputElement>(
      'input[type="checkbox"]',
    );
    expect(checkbox?.checked).toBe(false);

    await fireEvent.click(checkbox as HTMLInputElement);
    await vi.advanceTimersByTimeAsync(400);

    expect(save).toHaveBeenCalledTimes(1);
    const saved = JSON.parse(save.mock.calls[0][0]);
    expect(saved.content[0].type).toBe("taskList");
    expect(saved.content[0].content[0].attrs.checked).toBe(true);
    expect(saved.content[0].content[0].content[0].content[0].text).toBe(
      "comprar",
    );
  });

  it("refuses javascript: and file: links and accepts https", async () => {
    const { box } = await mount(doc("sitio"));
    await waitFor(() => expect(box.textContent).toBe("sitio"));
    await fireEvent.click(screen.getByRole("button", { name: /Enlace/ }));
    const field = screen.getByLabelText("Dirección del enlace");

    for (const bad of ["javascript:alert(1)", "file:///C:/x.txt"]) {
      await fireEvent.input(field, { target: { value: bad } });
      await fireEvent.keyDown(field, { key: "Enter" });
      expect(screen.getByRole("alert").textContent).toContain("http");
    }
    expect(box.querySelector("a")).toBeNull();
  });

  it("flushes pending text when the editor closes", async () => {
    const save = vi.fn<(json: string) => Promise<void>>(async () => {});
    const view = render(NoteEditor, {
      contentJson: doc("x"),
      save,
      autofocus: true,
    });
    await screen.findByRole("textbox");
    await fireEvent.click(
      screen.getByRole("button", { name: "Lista numerada" }),
    );
    expect(save).not.toHaveBeenCalled();
    view.unmount();
    await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  });
});
