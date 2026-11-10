# Config

Decision: [techstack](../techstack.md) (`confy` only, `figment` parked). Status: documented, not implemented.

1. One TOML file per user, same file for admin and user. Everything runs as the current desktop user.
2. Path comes from `confy` (via etcetera): `~/.config/handsfree/` on Linux, the platform equivalent elsewhere. No `/usr/share` or root layers.
3. Read and write through `confy::load` / `confy::store`. The config struct derives `Default`, `Serialize`, `Deserialize`.
4. Defaults are embedded in the binary: a TOML file in the owning crate's `resources/`, loaded with `include_str!`. No loose assets on disk.
5. The owning crate holds the struct and its default file. A crate that needs another crate's settings depends on that crate. A shared config crate is created only when two crates need the same struct.
6. Brand identity is not config: `brand/brand.toml`, embedded the same way.
7. Global hot keys are settings, added when the settings UI exists.
