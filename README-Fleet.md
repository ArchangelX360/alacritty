# Alacritty Fleet fork instructions

## Naming conventions

| Concept               | Naming convention                                             | Example                                  |
|-----------------------|---------------------------------------------------------------|------------------------------------------|
| Fleet fork branch [1] | `fleet/<upstream_tag_name>`                                   | `fleet/v0.12.0`                          |
| Fleet version tag     | `v<fork_version_of_package>-fleet.<incremental_build_number>` | `v1.18.0-rc3-fleet.0`, `v1.18.0-fleet.5` |

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
      of ["Publish on Fleet tag" job](https://jetbrains.team/p/fleet/automation/jobs/history/3J7qVd2zQgGA) (Job list
      can be found [here](https://jetbrains.team/p/fleet/automation/jobs/runs) in case link is broken)
- That's it! You should see your crate in
  the [`fleet-crates` repository](https://jetbrains.team/p/fleet/packages/crates/fleet-crates)

### Chose the correct next `incremental_build_number`

If this is the first release of this fork branch, simply chose `0`.
> Example, if we are on branch `fleet/v0.12.0` and `alacritty_terminal` package is in version `1.18.0` and no fork
> version have been released yet by Fleet developers, then your full tag name will be `v1.18.0-fleet.0`.

If not, then check the latest crate of this fork branch
in [`fleet-crates` repository](https://jetbrains.team/p/fleet/packages/crates/fleet-crates) and increase
its `incremental_build_number` by 1.
> Example, if we are on branch `fleet/v0.12.0` and `alacritty_terminal` package latest version is `1.18.0-fleet.0`, then
> its next version will be `1.18.0-fleet.1` so your full tag name will be `v1.18.0-fleet.1`.

## Create a new "Fleet fork branch" from an upstream release

- Chose the upstream release tag you want to fork from
- Create the branch from the upstream tag, following the [naming convention table](#naming-conventions)
- Cherry-pick all relevant Fleet commits of the previous fork branch (usually all of
  them), [the CI automation one](https://jetbrains.team/p/fleet/repositories/alacritty/revision/fe3d5aab9ae3bf96ea18491b5c076e58a419eb46)
  is mandatory
- Eventually, release a new version following [the version release instructions](#release-a-new-forked-version)
