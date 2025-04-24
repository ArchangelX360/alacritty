# Alacritty Fleet fork instructions

## Getting started

`alacritty_terminal` is a main crate used in Fleet. It depends on `alacritty/vte`, which located in 
separate repository. So, you need to init git submodules

```shell
git submodule init
git submodule update
```

To compile `alacritty_terminal` with local `vte` version change dependency in [Cargo.toml](alacritty_terminal/Cargo.toml)
```toml
vte = { path = "../alacritty_vte", default-features = false, features = ["ansi", "serde"] }
```

## Naming conventions

| Concept               | Naming convention                                                   | Example                                                   |
|-----------------------|---------------------------------------------------------------------|-----------------------------------------------------------|
| Fleet fork branch [1] | `fleet/<upstream_tag_name>`                                         | `fleet/v0.12.0`                                           |
| Fleet version tag     | `fleet/v<fork_version_of_package>-fleet.<incremental_build_number>` | `fleet/v1.18.0-rc3-fleet.0`, <br/>`fleet/v1.18.0-fleet.5` |

[1] feature branch forked from an upstream tag

## Release a new forked version

You pushed commits to one of our fork branches, and you want to release your changes, so you should follow the following
instructions:

- Think about what will be the tag name, following the [naming convention table](#naming-conventions)
    - You could check
      instructions [on how to choose the correct `incremental_build_number`](#chose-the-correct-next-incrementalbuildnumber)
- Tag the commit (_on a Fleet fork branch_) you want to release (e.g. a commit of `fleet/v0.12.0`)
- Wait for publishing to happen automatically
    - Follow progress
      of ["Publish alacritty_terminal" build](https://buildserver.labs.intellij.net/buildConfiguration/ijplatform_master_FleetPublishAlacrittyTerminalBuild)
- That's it! You should see your crate in
  the [`fleet-crates` repository](https://jetbrains.team/p/fleet/packages/crates/fleet-crates)

### Chose the correct next `incremental_build_number`

If this is the first release of this fork branch, simply chose `0`.
> Example, if we are on branch `fleet/v0.12.0` and `alacritty_terminal` package is in version `1.18.0` and no fork
> version have been released yet by Fleet developers, then your full tag name will be `fleet/v1.18.0-fleet.0`.

If not, then check the latest crate of this fork branch
in [`fleet-crates` repository](https://jetbrains.team/p/fleet/packages/crates/fleet-crates) and increase
its `incremental_build_number` by 1.
> Example, if we are on branch `fleet/v0.12.0` and `alacritty_terminal` package latest version is `1.18.0-fleet.0`, then
> its next version will be `1.18.0-fleet.1` so your full tag name will be `fleet/v1.18.0-fleet.1`.

## Create a new "Fleet fork branch" from an upstream release

- Chose the upstream release tag you want to fork from
- Create the branch from the upstream tag, following the [naming convention table](#naming-conventions)
- Cherry-pick all relevant Fleet commits of the previous fork branch (usually all of
  them)
- Eventually, release a new version following [the version release instructions](#release-a-new-forked-version)
