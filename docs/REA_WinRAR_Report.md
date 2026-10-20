# WinRAR 7.23 (x64) — Read-Only Reverse Engineering Analysis

Target: `C:\Program Files\WinRAR` (WinRAR 7.23.0, 64-bit, publisher win.rar GmbH)
Date: 2026-10-20
Method: metadata-only inspection. No raw byte dumps were loaded into the analysis context.

Tools used (all non-destructive):

| Tool | Purpose |
|---|---|
| `Get-ChildItem`, `Get-FileHash`, `Get-AuthenticodeSignature`, `VersionInfo` | Inventory, hashes, signatures, version resources |
| `dumpbin.exe` (VS 2022 14.44) `/headers /imports /dependents /exports` | PE headers, import/export tables |
| `rg -a -o` (ripgrep, ASCII + UTF‑16LE) | Targeted string-pattern checks (module names, algorithm identifiers, manifests) |
| `reg query` | Shell-extension / ProgID / uninstall registration |
| REA `inspect_pe_resources` (evidence `ev_317daa3c…ceeb`) | Resource tree of `Default.SFX` |
| `Rar.exe -?`, `UnRAR.exe`, `Rar.txt` | Documented CLI surface |
| `Rar.exe a / lt` in `%TEMP%` | Behavioural experiment on SFX layout (install dir untouched; temp files removed) |

---

## Tier 1 — What can be proven (direct evidence)

### 1.1 Directory inventory (32 files, all in one flat directory)

| File | Size | Kind | Signed | Description (VersionInfo / `Descript.ion`) |
|---|---:|---|---|---|
| `WinRAR.exe` | 3,364,888 | PE32+ x64, GUI | Yes (win.rar GmbH) | WinRAR GUI archive manager |
| `Rar.exe` | 846,352 | PE32+ x64, CUI | Yes | Console RAR (create + extract) |
| `UnRAR.exe` | 571,408 | PE32+ x64, CUI | Yes | Freeware extractor |
| `RarExt.dll` | 685,584 | PE32+ x64 DLL | Yes | Explorer shell extension (COM in‑proc) |
| `RarExt32.dll` | 578,072 | PE32 x86 DLL | Yes | Same, for 32‑bit hosts (OriginalFilename `RarExt.dll`) |
| `RarExtInstaller.exe` | 234,520 | PE32+ x64, GUI | Yes | Registers the Win11 sparse‑package context menu |
| `RarExtPackage.msix` | 24,437 | MSIX (ZIP) | Signed p7x | Sparse package manifest for Win11 context menu |
| `Resources.pri`, `RarExtLogo*.png` | small | data | – | Resources for the MSIX package |
| `7zxa.dll` | 239,128 | PE32+ x64 DLL | Yes (re‑signed by win.rar) | 7‑Zip 26.02 "Standalone Extracting Plugin" (Igor Pavlov) |
| `Default.SFX` / `Default32.SFX` | 480,768 / 386,048 | PE32+/PE32 GUI | **Not signed** | GUI RAR SFX stub x64 / x86 |
| `WinCon.SFX` / `WinCon32.SFX` | 421,376 / 339,456 | PE32+/PE32 CUI | **Not signed** | Console RAR SFX stub |
| `Zip.SFX` / `Zip32.SFX` | 495,104 / 350,208 | PE32+/PE32 GUI | **Not signed** | GUI ZIP SFX stub |
| `Uninstall.exe` | 423,440 | PE32+ GUI | Yes | Uninstaller |
| `rarnew.dat` / `zipnew.dat` | 24 / 22 bytes | data | – | Empty‑archive templates (for "New → WinRAR archive") |
| `RarFiles.lst` | 1,321 | text | – | Solid‑archive file ordering list |
| `rarreg.key` | 478 | text | – | License key (registered to a named user) |
| `Rar.txt`, `WinRAR.chm`, `WhatsNew.txt`, `ReadMe.txt`, `License.txt`, `Order.htm`, `Descript.ion`, `Uninstall.lst` | – | docs | – | Documentation / metadata |

Notable absences: no `RarLng.dll`, `*.lng` (English‑only build); no `UnRAR.dll` (the public SDK DLL is not shipped with the GUI product); no `rar.ini`/`WinRAR.ini` (settings live in the registry: `HKCU\Software\WinRAR\{ArcHistory, DialogEditHistory, FileList, General, Interface, Profiles, Setup}`).

All PE files share: MSVC linker 14.44, build timestamps 27 Jun 2026 (7zxa.dll: 25 Jun 2026), DEP/NX, ASLR + high‑entropy VA, Control Flow Guard (except `RarExtInstaller.exe`, `7zxa.dll`), `asInvoker` execution level, Common‑Controls v6 manifest, `dpiAware=true` (GUI binaries).

### 1.2 Separation of concerns

| Concern | Binary | Proof |
|---|---|---|
| GUI manager | `WinRAR.exe` | Subsystem = Windows GUI; imports 164 USER32 + 33 GDI32 + 37 gdiplus + COMCTL32 (`PropertySheetW`, `ImageList_*`, `CreateStatusWindowW`), UxTheme, dwmapi, COMDLG32 (`GetOpenFileNameW`, `ChooseFontW`), ole32 drag‑drop (`RegisterDragDrop`, `DoDragDrop`, `OleSetClipboard`), `Shell_NotifyIconW`, `SHAddToRecentDocs`. Registered as `shell\open\command` for ProgIDs `WinRAR`, `WinRAR.ZIP`, `WinRAR.REV` and in `App Paths`. |
| CLI create/extract | `Rar.exe` | Subsystem = Console; only 5 import DLLs (KERNEL32, USER32, ADVAPI32, SHELL32, POWRPROF); console APIs `SetConsoleCtrlHandler`, `SetConsoleMode`, `ReadConsoleW`, `WriteConsoleW`; help lists 23 commands incl. `a, d, f, u, m, rr, rv, s, k, ch, c, cw, rn, i`. |
| CLI extract‑only | `UnRAR.exe` | Same import profile as `Rar.exe` minus 7 KERNEL32 / 5 ADVAPI32 symbols (notably no `BackupRead/BackupSeek`, `CreateHardLinkW` retained, no `CryptAcquireContextW/CryptGenRandom`, no `GetFileSecurityW`). Help lists only `e, l, p, t, v, x`. |
| Explorer integration | `RarExt.dll` / `RarExt32.dll` | Exports exactly `DllCanUnloadNow`, `DllGetClassObject` (classic COM in‑proc server). CLSID `{B41DB860-64E4-11D2-9906-E49FADC173CA}` → `InprocServer32 = C:\Program Files\WinRAR\rarext.dll`, `ThreadingModel=Apartment`. Registered as `ContextMenuHandlers\WinRAR` under `*` and `Folder`, `DragDropHandlers\WinRAR` under `Folder` and `Drive`; CLSID `{B41DB860-8EE4-…}` registered as `WinRAR32` (32‑bit host variant). Imports `CreatePropertySheetPageW` (property‑sheet handler), `DragQueryFileW`, `ReleaseStgMedium`, `CoCreateFreeThreadedMarshaler`. |
| Win11 modern context menu | `RarExtInstaller.exe` + `RarExtPackage.msix` | MSIX `AppxManifest.xml`: `com:SurrogateServer` hosting class `b41db860‑…` from `RarExt.dll` (`ThreadingModel=STA`), `desktop4:FileExplorerContextMenus` verb `ATopItemWinRAR` on `*` and `Directory`, `runFullTrust`, `AllowExternalContent=true` (sparse package pointing at the install dir). Installer imports `CoCreateInstance`, `SHCreateStreamOnFileEx`, `SHChangeNotify` (package registration + shell refresh). |
| Foreign‑format decoder | `7zxa.dll` | Exports the 7‑Zip plug‑in ABI: `CreateDecoder, CreateObject, GetHandlerProperty(2), GetNumberOfFormats, GetNumberOfMethods, GetMethodProperty, GetHashers, GetIsArc, SetLargePageMode, SetCodecs`. Only `WinRAR.exe` and `RarExt.dll` contain the UTF‑16 string `7zxa.dll`; `Rar.exe`/`UnRAR.exe` do not. |
| SFX stubs | `*.SFX` | Valid PE images (own import tables and manifests, name `WinRAR SFX`) with **no exports** and **no Authenticode signature** (they must remain patchable: WinRAR imports `BeginUpdateResourceW / UpdateResourceW / EndUpdateResourceW` for icon/dialog replacement). `Default.SFX` resources: dialogs `STARTDLG, LICENSEDLG, GETPASSWORD1, REPLACEFILEDLG, RENAMEDLG, ASKNEXTVOL`; string tables 7–17; icon group 100 (7 images); 2 PNGs; RT_MANIFEST. |

### 1.3 Imported Windows APIs grouped by capability

**File I/O & NTFS metadata (Rar.exe / WinRAR.exe)**
`CreateFileW, ReadFile, WriteFile, SetFilePointer(Ex), SetEndOfFile, FlushFileBuffers, FindFirstFileExW/FindNextFileW, GetFileInformationByHandle, GetFileTime/SetFileTime, SetFileAttributesW, MoveFileW, DeleteFileW, RemoveDirectoryW, CreateDirectoryW, GetDiskFreeSpaceExW, GetVolumeInformationW, GetLongPathNameW/GetShortPathNameW, DeviceIoControl` (reparse points / symlinks `-ol`), `CreateHardLinkW` (`-oh`), `BackupRead/BackupSeek` (NTFS alternate streams `-os`, Rar.exe only), `GetFileSecurityW/SetFileSecurityW/GetSecurityDescriptorLength` (`-ow`), `GetCompressedFileSizeW` (WinRAR.exe), `SHFileOperationW` (`-dr` recycle‑bin delete).

**Threading & synchronisation (all three core binaries)**
`CreateThread, CreateEventW, SetEvent/ResetEvent, CreateSemaphoreW/ReleaseSemaphore, WaitForSingleObject, InitializeCriticalSection(AndSpinCount/Ex), Acquire/ReleaseSRWLockExclusive, SleepConditionVariableSRW, WakeAllConditionVariable, GetProcessAffinityMask, SetThreadPriority, SetPriorityClass` (`-ri`), `SetThreadExecutionState` (keep‑awake), TLS/FLS. WinRAR.exe adds `WaitForMultipleObjects, Suspend/ResumeThread, CreateMutexW, GetSystemInfo, GetPhysicallyInstalledSystemMemory, GlobalMemoryStatusEx` and named pipes (`CreateNamedPipeW, ConnectNamedPipe, PeekNamedPipe, WaitNamedPipeW`), plus `FindFirstChangeNotificationW` (watch extracted‑and‑opened files for modification).

**Memory**
`VirtualAlloc/VirtualFree/VirtualProtect, HeapAlloc/ReAlloc`; `AdjustTokenPrivileges + LookupPrivilegeValueW + LsaOpenPolicy/LsaAddAccountRights` (grant *SeLockMemoryPrivilege* for `-mlp` large pages, exactly as `Rar.txt` documents). `7zxa.dll` imports `GetLargePageMinimum`.

**Cryptography / randomness**
`ADVAPI32!CryptAcquireContextW, CryptGenRandom, CryptReleaseContext` (Rar.exe, WinRAR.exe delay‑load; absent from UnRAR.exe → only needed when *creating* encrypted archives, i.e. salt/IV generation). No `bcrypt.dll` / `ncrypt.dll` imports and no CryptoAPI hashing/cipher calls → **AES, SHA‑256, BLAKE2sp, CRC32 are implemented in‑binary**, consistent with the identifier strings found (`AES, SHA256, BLAKE2, CRC32`) and with the public UnRAR source. `Crypt32.dll` appears only as a UTF‑16 string (runtime `LoadLibrary`, purpose unproven; likely certificate/MOTW related).

**Power / shell / misc**
`POWRPROF!SetSuspendState`, `USER32!ExitWindowsEx` (`-ioff`), `ShellExecuteExW`, `SHGetSpecialFolderLocation`, `RegOpenKeyExW/RegQueryValueExW` (reads `HKCU\Software\WinRAR`), MAPI32.DLL string (`-ieml` e‑mail), `EventRegister/EventWriteTransfer` (ETW, WinRAR.exe).

**Runtime‑loaded module strings (UTF‑16, WinRAR.exe)**
`7zxa.dll, rarext.dll, rarext32.dll, RarExtInstaller.exe, Default.SFX, Default32.SFX, WinCon.SFX, WinCon32.SFX, Zip.SFX, Zip32.SFX, rarnew.dat, zipnew.dat, rarfiles.lst, rarlng.dll, winrar.lng, WinRAR.ini, rar.ini, winrar.chm, riched20.dll, MAPI32.DLL, EmbeddedBrowserWebView.dll, cabinet.dll, version.dll`, plus a list of AV scanner executables (`MpCmdRun.exe, avpui.exe, nod32.exe, clamscan.exe, …`) — the "Virus scan" feature — and browser names (`chrome.exe, firefox.exe, …`) used for Mark‑of‑the‑Web zone propagation (`-om`, string `Zone.Identifier`).

**Format identifier strings in WinRAR.exe**: `RAR5, RAR 4.x, Rar!, 7‑Zip, LZMA, LZMA2, PPMd, BZIP2, Deflate64, Delta, XZ, Zstandard, AES, BLAKE2, SHA256, CRC32`. `Rar.exe`/`UnRAR.exe` carry only `Rar!, SHA256, BLAKE2, CRC32, xz, x86` — the console tools do not reference the 7‑Zip method names.

### 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`)

- `Rar.exe`: 23 commands, 118 switches. Compression levels `-m0..5`, dictionary `-md` 128 KB … 64 GB (RAR7, non‑power‑of‑2 above 4 GB), threads `-mt1..64`, filters `-mc[D|E|L|X]` (delta, x86 E8/E9, long‑range, exhaustive), solid `-s[=…]`, recovery record `-rr[N]`, recovery volumes `-rv[N]`, volumes `-v<size>`, header encryption `-hp`, data‑only `-p`, hash `-htb|c` (BLAKE2sp or CRC32), quick‑open `-qo`, dedup references `-oi`, SFX `-sfx[name]`, stdin `-si`, NTFS extras `-os -ow -oh -ol`, MOTW `-om`, large pages `-mlp`.
- `UnRAR.exe`: 6 commands (`e l p t v x`), ~50 switches — strict subset.
- Verified by experiment: `rar a out.zip …` produces a **RAR5** container regardless of extension (`Details: RAR 5`, first bytes `Rar!`). ZIP *creation* is a `WinRAR.exe`‑only capability; `Rar.exe` creates RAR only.
- Verified by experiment: `rar a -sfx out_sfx.exe src\*` produced a file whose **first 480,768 bytes are byte‑identical to `Default.SFX`** and the RAR5 signature `Rar!\x1A\x07\x01\x00` sits at **offset 480,768** — the archive is simply appended to the stub. `UnRAR.exe t out_sfx.exe` → exit 0, `rar lt` reports `Details: RAR 5, SFX`.
- Verified by experiment: `-hp` yields `Details: RAR 5, encrypted headers`; listing without password exits **11** ("Incorrect password"). `-htb` lists a 256‑bit `BLAKE2` per file instead of `CRC32`. With ~16 KB of input the archiver auto‑shrinks the dictionary (`-md=128k` non‑solid, `-md=1m` solid), matching the manual.

---

## Tier 2 — What can be inferred (architectural deductions)

1. **One shared archiver core, three front‑ends.** `Rar.exe`, `UnRAR.exe`, `WinCon.SFX`, `Default.SFX` have near‑identical KERNEL32 import profiles, identical sections (`.text .rdata .data .pdata .didat .fptable .rsrc .reloc`), the same linker, the same manifest template and the same string set (`Rar!`, `SHA256`, `BLAKE2`, `CRC32`, `Zone.Identifier`). `UnRAR.exe` is the same code with the encoder, `CryptGenRandom`, `BackupRead` and security‑descriptor paths compiled out. The SFX stubs are the extractor core plus a tiny dialog layer (`STARTDLG`, `GETPASSWORD1`, `REPLACEFILEDLG`, `ASKNEXTVOL` …) — i.e. `WinCon.SFX ≈ UnRAR.exe` built with "find my own tail" start‑up logic. Nothing is shared as a DLL; everything is statically linked (no `UnRAR.dll`, no CRT DLL — the CRT is static too).

2. **Archive creation pipeline (RAR).** From switches, imports and behaviour: file enumeration (`FindFirstFileExW`, optional `RarFiles.lst` sort for solid mode) → per‑file metadata capture (times, attributes, optional ACL/ADS/links) → optional pre‑filters chosen per data type (delta channels, x86 E8/E9, long‑range match finder) → LZ + entropy coder with a sliding dictionary sized from `-md` and auto‑reduced to input size → checksum (CRC32 or BLAKE2sp) → optional AES encryption with per‑archive salt from `CryptGenRandom` and a PBKDF2‑style KDF (`SHA256` string present; KDF details documented publicly for RAR5) → block writer with optional recovery record (Reed‑Solomon‑class; name not present as a string) and volume splitting → optional quick‑open block at the end. `-mt` changes output bytes, so compression is parallelised at the match‑finder/block level, not just I/O.

3. **Extraction pipeline.** Signature scan (RAR5 `Rar!\x1A\x07\x01\x00`, RAR4 `Rar!\x1A\x07\x00`, and for SFX a search past the PE image — proven by the appended‑archive layout) → header parse (decrypt headers first when `-hp`) → per‑file decode → verify CRC32/BLAKE2 → restore metadata. The `-si` documentation (no backward seeks, no quick‑open, no multivolume from stdin) implies the normal path uses random‑access seeks and a read‑ahead buffer; the stdin path is a strictly sequential fallback.

4. **Threading model.** Events + semaphores + SRW locks + condition variables, `GetProcessAffinityMask`/`GetSystemInfo` for CPU counting, `-mt1..64` cap, `SetThreadPriority` for `-ri`. This is a hand‑rolled thread pool (worker threads signalled by events) rather than the Win32 ThreadPool API (no `CreateThreadpoolWork` import). `7zxa.dll` uses `_beginthreadex` + the same primitives, so multi‑threaded LZMA2/7z decoding is delegated to that plug‑in.

5. **WinRAR.exe ↔ 7zxa.dll.** WinRAR loads the 7‑Zip extraction plug‑in dynamically (string only, not in the import table) and drives it through 7‑Zip's COM‑style `CreateObject`/`IInArchive` ABI (`OLEAUT32` BSTR/VARIANT imports in `7zxa.dll`). That is how 7Z, XZ, LZMA2, PPMd, BZIP2, Deflate64, ZSTD names end up in `WinRAR.exe` strings while `Rar.exe` lacks them. The console tools therefore handle only RAR (and `UnRAR.exe` has no 7z/zip path at all).

6. **Shell extension design.** `RarExt.dll` is deliberately thin: 3 ADVAPI32 imports (registry read only), no crypto, no compression strings beyond `7zxa.dll` (for the "Extract here" dry listing / icon overlays), and it spawns `WinRAR.exe` (`ShellExecuteExW`, string `winrar.exe`) to do real work — a classic *out‑of‑process worker* pattern that keeps Explorer stable. The same DLL serves both the legacy `IContextMenu`/`IShellPropSheetExt`/drop‑handler registrations and the Win11 `IExplorerCommand` surrogate declared in the MSIX.

7. **GUI ↔ SFX authoring.** WinRAR imports `BeginUpdateResourceW/UpdateResourceW/EndUpdateResourceW` and `EnumResourceNamesW/Languages`; the SFX stubs are unsigned and contain replaceable icon group 100 and PNG 101/102. Inference: "SFX options → Module/Icon/Text" rewrites resources in a copy of the stub before appending the archive; signing the stubs would break this, hence they ship unsigned.

8. **Named pipes & change notifications in WinRAR.exe** are consistent with (a) the single‑instance / "open file from archive and watch for edits, then offer to update" workflow (`FindFirstChangeNotificationW`) and (b) IPC between an elevated helper and the UI (`CreateNamedPipeW`, `ConnectNamedPipe`), e.g. when extraction needs admin rights. Not proven which, but the API set matches both.

9. **Licensing.** `rarreg.key` is a small text file; `Rar.exe` prints "Registered to …" from it. No network/HTTP imports exist in any binary (no `WinHTTP`, `WinINet`, `ws2_32` in import tables), so validation is offline. The `ws2_32.dll`/`wintrust.dll` names seen in the SFX stubs are part of a long list of system DLL names that also includes `apphelp.dll, cryptbase.dll, propsys.dll, …` — this pattern matches a DLL‑planting hardening list (pre‑load or block known hijackable DLLs from the SFX's own directory), not functional networking.

---

## Tier 3 — What remains hidden (opaque internals)

| Area | Why it is not recoverable from surface inspection |
|---|---|
| RAR5/RAR7 LZ match‑finder, lazy/optimal parsing heuristics, Huffman table construction, PPMd/`-mcx` exhaustive‑mode internals | Only switch semantics (`-mc`, `-md`, `-m`) and memory estimates are documented. Code is statically linked, stripped and proprietary; the public UnRAR source covers decoding only, and the RAR *encoder* has never been published. |
| Long‑range (`-mcl`) match finder data structure (hash chains vs. suffix structures), memory ~7× dictionary | Only the documented memory multiplier is observable. |
| Exact filter‑selection heuristic (when delta/E8E9/audio filters are auto‑applied) | Manual says "RAR chooses modes automatically, based on data" — the decision function is internal. |
| Solid‑statistics reset policy beyond the documented modifiers; per‑file‑type `RarFiles.lst` interplay with `-ds` | Only the list format is visible. |
| Recovery‑record encoding parameters (RS code size, interleave, hash of blocks) | No identifiers in strings; only `-rr[N]` percentages are documented. |
| Thread partitioning scheme (how `-mt` splits the match finder; why output differs per thread count) | Requires disassembly/decompilation of the encoder. |
| Quick‑open block layout decisions (`-qo` heuristics), `-oi` identical‑file detection thresholds | Only switch help. |
| Password KDF iteration count and key cache behaviour, header‑encryption check value | RAR5 public spec documents PBKDF2‑HMAC‑SHA256 with 2^N iterations; the exact N and the password‑cache policy of this build were not verified here. |
| 7zxa.dll feature set actually invoked by WinRAR (which formats are routed to 7zxa vs. native code, e.g. ZIP/TAR/GZ) | Would need runtime tracing (not performed; read‑only static scope). |
| License key validation algorithm (`rarreg.key` → "Registered to") | Offline, in‑binary; not inspected. |
| Purpose of the `.fptable` section and the runtime use of `Crypt32.dll`, `EmbeddedBrowserWebView.dll`, `version.dat` | Names only; no behavioural evidence gathered. |
| GUI internal state machine (window procedures, background worker ↔ UI progress protocol) | Would require decompilation of `WinRAR.exe`; out of scope for metadata‑only inspection. |

---

## Evidence ledger (summary)

| Claim | Evidence type | Source |
|---|---|---|
| Inventory, sizes, versions, hashes, signatures | observed | PowerShell listing / `Get-AuthenticodeSignature` |
| Subsystems, linker, mitigations, sections, import DLLs, import/export names | observed | `dumpbin /headers /imports /dependents /exports` |
| COM registration, ProgIDs, uninstall, HKCU settings keys | observed | `reg query` |
| MSIX sparse‑package manifest | observed | `AppxManifest.xml` inside `RarExtPackage.msix` |
| SFX dialog/icon resources | observed | REA `inspect_pe_resources` evidence `ev_317daa3c94b20d6704e729bd5ef6bc6b9a026cd5be0e99b51a4b19cd3cf2ceeb` |
| SFX = stub ‖ archive; `rar a *.zip` → RAR5; `-hp` exit 11; auto dictionary shrink | observed (behavioural, temp dir) | `Rar.exe a / lt / t` |
| Algorithm names, runtime module names, AV/browser lists | observed strings (not executed) | `rg -a -o` ASCII/UTF‑16 |
| Shared core, pipeline shapes, thread pool, 7zxa delegation, SFX authoring via resource update | inferred | correlation of the above |
| Encoder heuristics, RS parameters, KDF constants, license check | unknown | not recoverable statically without decompilation |
