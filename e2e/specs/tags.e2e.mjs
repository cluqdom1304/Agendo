// Tags (F9): creating one from the sidebar. Written to be run by a person on Windows 11 with
// tauri-driver (see e2e/README.md); the agent cannot run it.
describe("etiquetas", () => {
  it("crea una etiqueta desde la barra lateral", async () => {
    const add = await $('button[aria-label="Nueva etiqueta"]');
    await add.waitForClickable();
    await add.click();
    const name = await $("input[data-autofocus]");
    await name.waitForDisplayed();
    await name.setValue("Matemáticas");
    await $("button=Guardar").click();

    const toggle = await $('[aria-label="Mostrar tareas de Matemáticas"]');
    await toggle.waitForDisplayed({ timeoutMsg: "la etiqueta no aparece" });
  });

  it("rechaza un nombre repetido", async () => {
    await $('button[aria-label="Nueva etiqueta"]').click();
    const name = await $("input[data-autofocus]");
    await name.waitForDisplayed();
    await name.setValue("Matemáticas");
    await $("button=Guardar").click();
    const error = await $("*=Ya existe una etiqueta con ese nombre.");
    await error.waitForDisplayed();
    await browser.keys("Escape");
  });
});
