# M3 source-extension gate — revision 1

Assessment date: 2026-10-31. Accepted base:
`86b103c1ed91449c227f15b980bfce179d786135`, tree
`40fd9b2bd6c40d5cf5762567f68781d90be1071e`.

**Disposition: bounded pass with the owner-accepted BOW-M3-G01 exception and the
limitations below.** This permits the separately authorized M3 implementation to
proceed; it is not M3 acceptance, a public release, universal extension uniqueness,
security-product qualification, or new platform support.

The [M3 order](https://github.com/gregkrsak/klyxr/pull/101#issuecomment-6549034590)
requires this assessment before source renaming. The
[owner disposition](https://github.com/gregkrsak/klyxr/pull/101#issuecomment-6549608191)
accepts the VirtualBow collision while retaining the remaining bounded gate.
Canonical spelling is lowercase `.bow`. The legacy-filename discussion creates
no port, filename alias, discovery rule, or second source suffix assignment.

## Scope and decision basis

The compiler opens an explicitly supplied path. It does not call a desktop file
handler, discover a same-stem sibling, infer authority from a suffix, load a
configuration file, or register an editor/OS association. Both M2 commands compile
the same implementation. The migration must preserve those boundaries and
explicit `.klx` input, including other already accepted explicit filename forms.

The bounded pass rests on ordinary filename/byte transport, unchanged explicit
source ingestion, no internal destination collision, and the absence of an
additional demonstrated conflict that prevents this source-only use. It does not
rest on absence from a registry, assumed automatic highlighting, or assumed mail,
DLP or antivirus clearance. Downstream policies and desktop choices can differ.

## Known collisions and the accepted exception

**BOW-M3-G01 remains a real collision.** VirtualBow uses `.bow` for an unrelated
bow-model format. Its current v0.10.0 release was published 2026-08-29 at tag commit
`233dde4123fe8ef21a8b171fd8abe0286d5abb0e` and supplies Windows, macOS and Linux
packages. Primary sources examined:

- [Current model manual](https://www.virtualbow.org/files/user-manual/model-editor.html)
  and its [pinned source](https://github.com/bow-simulation/virtualbow/blob/233dde4123fe8ef21a8b171fd8abe0286d5abb0e/docs/user-manual/src/model-editor.md).
- [Windows installer](https://github.com/bow-simulation/virtualbow/blob/233dde4123fe8ef21a8b171fd8abe0286d5abb0e/gui/platforms/windows/setup.iss),
  blob `053a96afc873c68adbeac893d330889f38594174`: its FileAssociation task maps
  `.bow` to VirtualBowModelFile and a shell-open command for virtualbow-gui.exe.
- [macOS Info.plist](https://github.com/bow-simulation/virtualbow/blob/233dde4123fe8ef21a8b171fd8abe0286d5abb0e/gui/platforms/macos/Info.plist),
  blob `beea1afcc829d2b03404afd1230a4c6d73a88138`: declares extension bow with
  the Editor document role. These declarations were inspected, not installed.
- [Linux desktop entry](https://github.com/bow-simulation/virtualbow/blob/233dde4123fe8ef21a8b171fd8abe0286d5abb0e/gui/platforms/linux/virtualbow-gui.desktop)
  declares application/json. That file alone does not prove every Linux system's
  association or establish a suffix-specific MIME registration.

Greg accepts coexistence, not an assertion that these associations are absent.
A desktop open can select VirtualBow according to the local installation and
preferences. Use the explicit Bowfin command/path or an explicitly chosen editor.
M3 will neither take over associations nor change OS/editor settings.

Other documented reuse was also retained: a scientific sparse-matrix bag-of-words
format in the [Noble Lab project](https://github.com/Noble-Lab/lda_matrix_prior/blob/a2233ca363e65aead3d417b1d614e4912466153e/README.md)
(2023-03-08 revision; README blob
`09120bc0fe83daa377d83758f76217ab0d3fa6e7`) and historical BOXP workspace usage in
the [project's 2004-10-22 news](https://sourceforge.net/p/boxp/news/).
Neither examined source establishes another present desktop handler or proves
that current security products block benign `.bow` source by suffix. No such
security claim is made. Nonexclusive format reuse does not alter explicit-path
compiler ingestion; no alternate suffix was selected.

## Bounded platform and tooling matrix

All online evidence below was retrieved on 2026-10-31. Source inspection and
documented behavior are distinct from executing a product on a host.

| Surface | Evidence / actual check | Result and limitation |
| --- | --- | --- |
| Windows naming | [Win32 naming rules](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file), updated 2024-08-28 | `.bow` introduces no forbidden filename character; this is a naming inference. Reserved names, path limits and case-insensitivity still apply. No Windows filesystem or desktop test was available. |
| macOS naming and associations | [Current Mac extension guide](https://support.apple.com/en-hk/guide/mac-help/mchlp2304/mac) plus pinned VirtualBow metadata | Extensions affect display and association. Native filesystem/Launch Services tests were unavailable. Some later Apple fetches returned HTTP 429; no blanket Apple qualification is claimed. |
| Linux filesystem / case | Linux x86-64, kernel 6.18.57, Python 3.12.14; temporary `gate.bow` and `gate.klx` containing the exact public baseline battery_ok source | Both names and identical 359 source bytes were read successfully. Source SHA-256: `27bb129d87ebeb7dd670cf0116e8a0f67d25dd4ab6e8396aa8e9dc453f82c408`. All 75 proposed repository destinations are unique even under case folding and collide with no tracked destination. This does not qualify every filesystem. |
| Archives available locally | Python 3.12.14 ZIP/DEFLATE and TAR/GZIP round trips with both exact temporary names | Both member names and source bytes were preserved. Native Windows/macOS archive runs were unavailable. [Microsoft ZIP documentation](https://support.microsoft.com/en-us/windows/experience/storage-filemanagement/zip-and-unzip-files) documents generic packing/extraction, not an observed `.bow` run. [Apple ZIP guide](https://support.apple.com/en-euro/guide/mac-help/mchlp2528/27/mac/27) full retrieval was rate-limited; no native result is inferred. |
| Antivirus | [Defender scan documentation](https://learn.microsoft.com/en-us/defender-endpoint/configure-advanced-scan-types-microsoft-defender-antivirus), updated 2026-09-15 | Scanning is content/policy dependent. No Defender or other endpoint scanner was run. No source or private material was submitted to an external scanner; no malware-clearance claim. |
| DLP | [Purview endpoint settings](https://learn.microsoft.com/en-us/purview/dlp-configure-endpoint-settings), updated 2026-06-26 | Unsupported-extension and filename policies can restrict files. Actual tenant/device checks were unavailable. Plaintext does not bypass enterprise policy. This remains a disclosed transport constraint, not not-applicable. |
| Email | [Gmail blocked files](https://support.google.com/mail/answer/6590?hl=en), [Outlook blocked attachments](https://support.microsoft.com/en-us/outlook/blocked-attachments-in-outlook), [Workspace attachment rules](https://knowledge.workspace.google.com/admin/gmail/advanced/filter-messages-with-attachments?rd=1) | `.bow` was absent from the fetched default blocked lists; custom/content/archive rules still matter. No message was sent and no delivery claim is made. |
| GitHub file classification | [GitHub language documentation](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-repository-languages); complete Linguist languages.yml inspected at `73ee9a3b277f398c37ca4ab0dea0193849de6460`, blob `3eabbcc5341884422881be4517152ec2759bdc08` | No exact `.bow`, Bowfin or Klyxr entry in that snapshot. It does not prove the deployed revision, every heuristic or a highlighting guarantee. |
| GitHub fences | [Code-block documentation](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/creating-and-highlighting-code-blocks) | The `bowfin` fence is a project-facing language label. Changing it does not install a grammar; source code inside fences is unchanged. |
| GitLab | [Highlighting documentation](https://docs.gitlab.com/user/project/repository/files/highlighting/) and [development guide](https://docs.gitlab.com/development/fe_guide/source_code/blob_syntax_highlighting/) | Server/browser highlighting and Monaco Web IDE are different consumers. No GitLab deployment or actual rendering session was available. No GitLab facility is created. |
| VS Code | [Language overview](https://code.visualstudio.com/docs/languages/overview), dated 2026-10-28; official source at `235ceeb5f82f09812ec72c905c2a9ef6f04bba1e` | Unknown-language plaintext behavior is documented. Bounded upstream search/fetch found `.bowerrc`/Bower hits but no exact `.bow` extension; `.bowerrc` is not `.bow`. User extensions/associations and actual desktop operation are untested. |
| Sublime Text | [Syntax documentation](https://www.sublimetext.com/docs/syntax.html); official Packages tree at `722fa65e600b62812373b6f449ef9afa5f80b273`, 1,897 paths plus bounded code search | No exact `.bow` association was found in those official-source checks. First-line rules and third-party/user packages can change mode. No desktop instance was installed or tested. |
| Tree-sitter | [Official highlighting documentation](https://tree-sitter.github.io/tree-sitter/3-syntax-highlighting.html); upstream identity `ef95bde0db0ddb83db7a099e4a5328a769775afd` | File types belong to per-grammar configuration, not a universal suffix registry. No parser or editor package is introduced. No global grammar-ecosystem absence or execution claim. |
| Local MIME | file 5.45 and xdg-mime query on the exact temporary source; Python MIME lookup | Both file tools returned text/plain; Python guess_type returned no type. Local evidence does not establish global MIME registration. |
| HTTP defaults | Python 3.12.14 SimpleHTTPRequestHandler on loopback only; HTTP GET returned 200, application/octet-stream and exact 359 bytes | Source transport worked, but inline text/highlighting was not automatic. Server closed after the check. This is not production-hosting qualification. |
| NGINX / Apache | [NGINX types/default_type](https://nginx.org/en/docs/http/ngx_http_core_module.html#types), [Apache mod_mime](https://httpd.apache.org/docs/2.4/mod/mod_mime.html); shipped tables inspected at NGINX `67290bfea61c81347464539f5e6d0a7cebec1fc8` (blob `1c00d701ae67050fc06092b5028bdad063a86c57`) and Apache `92e766e7873fb4f098c46ab140cc0ed27ef6aa6a` (blob `b94996799e2ea5d253e358498988f0120e0e0942`) | Neither inspected table has an exact `.bow` entry. Type/handler behavior depends on actual configuration. Neither server was installed or run; future source hosting must set appropriate metadata deliberately. |

## Existing project facilities and limits

Tracked-path inspection at the accepted base found no VS Code settings package,
Sublime syntax package, tree-sitter grammar/configuration or `.gitattributes`
integration. Those absent project facilities need no invented replacement. This
does not describe all external installations. Existing exact consumers are CLI
paths, Rust harness includes/fixtures, workflow arguments, runnable documentation
links and fence labels; they are the M3 migration population.

No native legacy-host support, 8.3 aliases, source discovery, file association
installation, MIME registration, language service, registry publication or public
web server is part of this gate. Windows/macOS runtime, endpoint security, tenant
DLP, mail delivery and actual editor/GitLab sessions remain unavailable evidence,
not not-applicable and not claimed passed. No additional demonstrated issue
prevents bounded explicit-path adoption after G01's owner disposition.

The 75 source bodies must retain their exact baseline blobs. The implementation
must independently test supported and deliberately rejected corpus cases, both
commands, explicit legacy/current paths, same-stem distinct files, missing paths,
diagnostic spans and actual-byte limits. Those forthcoming Actions results are
implementation evidence, not retroactively claimed as part of this pre-rename
static gate. No local Rust was run. No software, associations, editor settings,
external services, repository settings or security exclusions were installed or
changed for this assessment.
