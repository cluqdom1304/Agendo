# Pruebas E2E (F6-06)

WebdriverIO + `tauri-driver`. Solo Windows; las ejecuta una persona (`[H]`).

## Requisitos

```powershell
cargo install tauri-driver --locked
```

Además, `msedgedriver.exe` de la **misma versión mayor** que WebView2 instalado
([descarga](https://developer.microsoft.com/microsoft-edge/tools/webdriver/)) en el `PATH`.
Comprueba la versión de WebView2 en «Configuración → Aplicaciones» o con
`(Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}').pv`.

## Ejecución

```powershell
npm run e2e
```

`onPrepare` compila la versión release (`tauri build --no-bundle`) y cada sesión arranca la
aplicación con `MIAGENDA_DATA_DIR` apuntando a una carpeta temporal vacía: los datos reales del
usuario no se tocan y la carpeta se borra al terminar.
