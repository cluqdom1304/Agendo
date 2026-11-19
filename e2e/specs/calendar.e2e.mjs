// Calendar view (F6): switching to it and creating a task from the keyboard. Written to be run by
// a person on Windows 11 with tauri-driver (see e2e/README.md); the agent cannot run it.
describe("calendario", () => {
  it("cambia al calendario y muestra la cuadrícula de la semana", async () => {
    const calendar = await $(
      '//*[@role="radio" and normalize-space()="Calendario"]',
    );
    await calendar.waitForClickable();
    await calendar.click();
    const grid = await $('[role="grid"][aria-label="Horario de la semana"]');
    await grid.waitForDisplayed();
  });

  it("crea una tarea con hora desde una columna con el teclado", async () => {
    const column = await $('[role="gridcell"][data-date]');
    await column.waitForDisplayed();
    await column.click();
    await browser.keys("Enter");

    const title = await $('input[aria-label="Título de la nueva tarea"]');
    await title.waitForDisplayed();
    await title.setValue("Tutoría de cálculo");
    await browser.keys("Enter");

    const block = await $("[data-block]");
    await block.waitForDisplayed({ timeoutMsg: "la tarea no aparece" });
    expect(await block.getText()).toContain("Tutoría de cálculo");
  });

  it("pasa a la vista de mes", async () => {
    const month = await $('//*[@role="radio" and normalize-space()="Mes"]');
    await month.waitForClickable();
    await month.click();
    const grid = await $('[aria-label="Calendario del mes"]');
    await grid.waitForDisplayed();
  });
});
