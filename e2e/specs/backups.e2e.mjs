// Backups and import in the settings dialog (F14). The native file dialogs cannot be driven by
// WebDriver, so this only checks that the section and its actions are there. Written to be run by
// a person on Windows 11 with tauri-driver (see e2e/README.md); the agent cannot run it.
describe("copias de seguridad", () => {
  it("muestra las acciones de copia y restauración en los ajustes", async () => {
    await $('button[aria-label="Ajustes"]').click();
    await $('[role="dialog"]').waitForDisplayed();
    await $("*=Copias de seguridad").waitForDisplayed();
    await $("button=Crear copia…").waitForDisplayed();
    await $("button=Restaurar copia…").waitForDisplayed();
    await browser.keys("Escape");
  });

  it("ofrece la copia automática desactivada, diaria o semanal", async () => {
    await $('button[aria-label="Ajustes"]').click();
    await $('[role="dialog"]').waitForDisplayed();
    for (const mode of ["Desactivada", "Cada día", "Cada semana"]) {
      await $(`//*[normalize-space()="${mode}"]`).waitForDisplayed();
    }
    await browser.keys("Escape");
  });
});
