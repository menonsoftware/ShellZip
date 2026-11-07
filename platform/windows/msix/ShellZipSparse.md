# Sparse MSIX for Windows 11 compact menus

Windows 11 only shows sparse-package `IExplorerCommand` handlers on the compact menu. ShellZip’s `IContextMenu` DLL remains registered for Windows 10 and for **Show more options**.

## Outline

1. Build a sparse signed package that declares the ShellZip CLSID as an Explorer command provider.
2. `Add-AppxPackage -Path ShellZipSparse.msix` as the current user (developer mode or a trusted certificate).
3. Point the package at the already-installed `shellzip_shell.dll`.

Ship this package from a future release job when a code-signing certificate is available. Until then, `--integrate` is enough on Windows 10 and for the classic menu on Windows 11.
