# Sparse MSIX for Windows 11 compact menus

Windows 11 only shows sparse-package `IExplorerCommand` handlers on the compact menu for unpackaged (Velopack / portable) layouts. ShellZip’s `IContextMenu` DLL remains registered for Windows 10 and for **Show more options** after `shellzip --integrate`.

The Store MSIX built by `xtask msix-pack` already embeds `shellzip_shell.dll`, declares `com:SurrogateServer` for CLSID `{B8D4F6C2-5E3A-4B02-AF7D-9C2E1F3A4B56}`, and registers `desktop4:FileExplorerContextMenus` on `*`, `Directory`, and `Directory\Background`. Store users therefore get the cascaded Add / Extract menu on the compact context menu without a sparse package.

## Outline (future, Velopack / portable)

1. Reuse the Store `IExplorerCommand` implementation already in `shellzip_shell.dll`.
2. Build a sparse signed package that declares the Explorer-command CLSID, with `AllowExternalContent` pointing at the installed `shellzip_shell.dll`.
3. Keep the classic `{A7C3E5B1-4D2F-4A91-9E6C-8B1F0D2E3A45}` in-proc registration for Windows 10 and **Show more options**.

Until then, Velopack users get the cascaded menu under **Show more options**; Store users get the compact-menu cascade from the full-trust package plus the static Extract verbs under **Show more options**.
