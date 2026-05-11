# Eggs

An egg is one package of configuration, typically for one single application.
For example, your editor configuration `~/.config/nvim` would likely be one egg called `nvim`.

According to your `yolk.rhai`, these get deployed on your system, and may contain some templated files.

## Egg Configuration

Eggs are defined in `yolk.rhai` with various configuration options:

```rhai
feh: #{
    targets: "~/.config/feh",
    enabled: true,
},

vicinae: #{
    targets: "~/.config/vicinae",
    enabled: true,
    unsafe_shell_hooks: helper::arch_prescripts("vicinae"),
    depends: ["feh", "fcitx5"],
},
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `targets` | String or Map | Required | Deployment target(s) |
| `enabled` | Boolean | `true` | Whether to deploy this egg |
| `main_file` | String | None | Main file for editing |
| `strategy` | String | `"put"` | Deployment strategy (`put` or `merge`) |
| `templates` | Array | `[]` | List of templated files |
| `unsafe_shell_hooks` | Map | `{}` | Shell scripts to run on deploy/undeploy |
| `depends` | Array | `[]` | List of egg names this egg depends on |

## Dependencies

You can declare dependencies between eggs using the `depends` field:

```rhai
vicinae: #{
    targets: "~/.config/vicinae",
    depends: ["feh", "fcitx5"],
},
```

Dependencies are resolved using a DAG (Directed Acyclic Graph) with topological sorting.
This ensures that:

- Dependencies are always deployed **before** the dependent egg
- Circular dependencies are detected and reported as errors
- The execution order respects all dependency relationships

### Circular Dependency Detection

If you create a circular dependency (A depends on B, B depends on A), yolk will report an error:

```
Error: Circular dependency detected for 'a': a -> b -> a
```

### Execution Order

Use `yolk list` to see the dependency graph and execution order:

```bash
$ yolk list
├─ ✓ niri
│   ├─ ✓ gruvbox
│   └─ ✓ DankMaterialShell
├─ ✗ alacritty
├─ ✓ anki
└─ ✓ zen-browser

Legend: ✓ enabled, ✗ disabled, ? unreachable
```

Disabled eggs (with `enabled: false`) are still tracked in the dependency tree but won't be deployed.

### Diamond Dependencies

Yolk correctly handles diamond dependencies where multiple paths converge:

```
    a
   / \
  b   c
   \ /
    d
```

In this case, `a` will be deployed first, then `b` and `c` (in any order), and finally `d`.
