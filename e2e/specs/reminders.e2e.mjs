// Reminders of a task (F13): opening the panel under its row. Written to be run by a person on
// Windows 11 with tauri-driver (see e2e/README.md); the agent cannot run it.
describe("recordatorios", () => {
  it("crea una tarea y abre su panel de recordatorios", async () => {
    const create = await $("button*=Nueva tarea");
    await create.waitForClickable();
    await create.click();
    const input = await $("[data-title-input]");
    await input.waitForDisplayed();
    await input.setValue("Llamar al banco");
    await browser.keys("Enter");

    const open = await $(
      'button[aria-label^="Recordatorios de «Llamar al banco»"]',
    );
    await open.waitForClickable();
    await open.click();
    const heading = await $("//*[normalize-space()='Recordatorios']");
    await heading.waitForDisplayed();
  });

  it("cierra el panel al volver a pulsar el botón", async () => {
    const open = await $(
      'button[aria-label^="Recordatorios de «Llamar al banco»"]',
    );
    await open.click();
    const heading = await $("//*[normalize-space()='Recordatorios']");
    await heading.waitForDisplayed({ reverse: true });
  });
});
