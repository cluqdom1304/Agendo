describe("tareas", () => {
  it("abre la ventana con la barra de título de la aplicación", async () => {
    const title = await $("header");
    await title.waitForDisplayed();
    expect(await title.getText()).toContain("MiAgendaApp");
  });

  it("crea una tarea desde el estado vacío y la deja en la lista", async () => {
    const create = await $("button*=Nueva tarea");
    await create.waitForClickable();
    await create.click();

    const input = await $("[data-title-input]");
    await input.waitForDisplayed();
    await input.setValue("Entregar práctica");
    await browser.keys("Enter");

    const rows = await $$('[role="row"][data-task-row]');
    await browser.waitUntil(async () => (await rows.length) >= 1, {
      timeoutMsg: "la tarea no aparece en la lista",
    });
    expect(await $("[data-task-row] input").getValue()).toBe(
      "Entregar práctica",
    );
  });

  it("encuentra la tarea con el buscador (Ctrl+K)", async () => {
    await browser.keys(["Control", "k"]);
    const query = await $('[role="combobox"]');
    await query.waitForDisplayed();
    await query.setValue("práctica");
    const option = await $('[role="option"]');
    await option.waitForDisplayed();
    expect(await option.getText()).toContain("Entregar práctica");
    await browser.keys("Escape");
    await query.waitForDisplayed({ reverse: true });
  });
});
