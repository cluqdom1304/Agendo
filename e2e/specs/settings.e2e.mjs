describe("ajustes", () => {
  it("abre el diálogo de ajustes y lo cierra con Escape", async () => {
    const open = await $('button[aria-label="Ajustes"]');
    await open.waitForClickable();
    await open.click();
    const dialog = await $('[role="dialog"]');
    await dialog.waitForDisplayed();
    expect(await dialog.getAttribute("aria-modal")).toBe("true");
    await browser.keys("Escape");
    await dialog.waitForDisplayed({ reverse: true });
  });

  it("cambia al tema oscuro y luego al claro", async () => {
    await $('button[aria-label="Ajustes"]').click();
    await $('[role="dialog"]').waitForDisplayed();

    await $('//button[@role="radio" and normalize-space()="Oscuro"]').click();
    await browser.waitUntil(
      async () =>
        (await browser.execute(
          () => document.documentElement.dataset.theme,
        )) === "dark",
    );

    await $('//button[@role="radio" and normalize-space()="Claro"]').click();
    await browser.waitUntil(
      async () =>
        (await browser.execute(
          () => document.documentElement.dataset.theme,
        )) === "light",
    );
    await browser.keys("Escape");
  });

  it("el inicio con Windows está desactivado por defecto", async () => {
    await $('button[aria-label="Ajustes"]').click();
    const toggle = await $('input[role="switch"]');
    await toggle.waitForDisplayed();
    expect(await toggle.isSelected()).toBe(false);
    await browser.keys("Escape");
  });
});
