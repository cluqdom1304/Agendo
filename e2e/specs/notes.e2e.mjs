// Notes, search across notes and the import/export section (F15-03). Written to be run by a
// person on Windows 11 with tauri-driver (see e2e/README.md); the agent cannot run it.
describe("notas", () => {
  it("cambia a Notas con Ctrl+Mayús+2 y crea una nota con título", async () => {
    await browser.keys(["Control", "Shift", "2"]);
    const create = await $('button[aria-label="Nueva nota"]');
    await create.waitForClickable();
    await create.click();

    const title = await $('input[aria-label="Título de la nota"]');
    await title.waitForDisplayed();
    await title.setValue("Apuntes de álgebra");
    await browser.keys("Tab");

    await browser.waitUntil(
      async () => (await $$("[role='option'], [role='listitem']").length) >= 1,
      { timeoutMsg: "la nota no aparece en la lista" },
    );
  });

  it("encuentra la nota con el buscador (Ctrl+K)", async () => {
    await browser.keys(["Control", "k"]);
    const query = await $('[role="combobox"]');
    await query.waitForDisplayed();
    await query.setValue("algebra");
    const option = await $('[role="option"]');
    await option.waitForDisplayed();
    expect(await option.getText()).toContain("Apuntes de álgebra");
    await browser.keys("Escape");
    await query.waitForDisplayed({ reverse: true });
  });

  it("muestra la sección de exportar e importar en los ajustes", async () => {
    await $('button[aria-label="Ajustes"]').click();
    await $('[role="dialog"]').waitForDisplayed();
    for (const label of [
      "Exportar notas a Markdown…",
      "Exportar notas a HTML…",
      "Exportar todo a JSON…",
      "Importar carpeta de Markdown o texto…",
      "Importar desde Evernote…",
    ]) {
      expect(await $(`button=${label}`).isDisplayed()).toBe(true);
    }
    await browser.keys("Escape");
  });
});
