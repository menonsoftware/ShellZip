# Sparse MSIX for Windows 11 compact menus

Windows 11 only shows sparse-package `IExplorerCommand` handlers on the compact menu. ShellZip’s `IContextMenu` DLL remains registered for Windows 10 and for **Show more options** after `shellzip --integrate` (Velopack / portable).

The Store MSIX built by `xtask msix-pack` registers archive types and static Extract verbs. It does not yet declare `desktop4:FileExplorerContextMenus` or ship a COM surrogate for the compact menu.

## Outline (future)

1. Add `IExplorerCommand` to `shellzip_shell.dll` (or a thin companion).
2. Build a sparse signed package that declares the ShellZip CLSID as an Explorer command provider, or extend the Store package with the same declarations.
3. Point the package at the installed `shellzip_shell.dll` (`AllowExternalContent`) for Velopack layouts, or embed the DLL for Store layouts.

Until then, Velopack users get the cascaded menu under **Show more options**; Store users get Open and the static Extract verbs from the full-trust package manifest.
