+++
title = "Plan Settings"
description = "Define basic information about your artifact with plan settings"

[menu]
  [menu.habitat]
    title = "Plan Settings"
    identifier = "habitat/plans/plan-settings"
    parent = "habitat/plans"
    weight = 20

+++

The following settings are defined at the beginning of your plan. They specify basic information about your plan such as name, version, and dependencies.

> **Note**: We are using Bash syntax here, but Powershell plans use the same variable names and should conform to standard Powershell language rules (ie. `$` before variable names and quoting string values).

## General Settings

**pkg_name**
: _Required_. Sets the name of the package. This will be used along with `pkg_origin`, and `pkg_version` to define the fully-qualified package name, which determines where the package is installed to on disk, how it is referred to in package metadata, and so on. A `pkg_name` can contain upper and lowercase letters, numbers, dashes, and underscores.

```bash
pkg_name=zlib
```

**pkg_origin***
: _Required_ unless overridden by the `HAB_ORIGIN` environment variable. The origin is used to denote a particular upstream of a package. A `pkg_origin` can contain upper and lowercase letters, numbers, dashes, and underscores.

```bash
pkg_origin=Habitat
```

**kg_version**
: _Optional_ (Required unless `pkg_version()` function is defined). Sets the version of the package.

```bash
pkg_version=1.2.8
```

**pkg_maintainer**
: _Optional_. The name and email address of the package maintainer.

```bash
pkg_maintainer="Your Name <someone@example.com>"
```

**pkg_license**
: _Optional_. An array of [valid software licenses](https://spdx.org/licenses/) that relate to this package.

```bash
pkg_license=('Apache-2.0')
```

> **Note**: If your package has a custom license, use a string literal matching the title of the license. For example, you'll see `pkg_license=('Boost Software License')` for the `cmake` plan.

**pkg_source**
: _Optional_. A URL that specifies where to download an external source from. Any valid `wget` url will work. Typically, the relative path for the URL is partially constructed from the `pkg_name` and `pkg_version` values; however, this convention is not required.

```bash
pkg_source=http://downloads.sourceforge.net/project/libpng/$pkg_name/${pkg_version}/${pkg_name}-${pkg_version}.tar.gz
```

**pkg_filename**
: _Optional_. The resulting filename for the download, typically constructed from the `pkg_name` and `pkg_version` values.

```bash
pkg_filename=${pkg_name}-${pkg_version}.tar.gz
```

**pkg_shasum**
: _Required_ if a valid URL is provided for `pkg_source` or unless **do_verify()** is overridden. The value for `pkg_shasum` is a sha-256 sum of the downloaded `pkg_source`.

If you do not have the checksum, you can easily generate it by downloading the source and using the `sha256sum` or `gsha256sum` tools. Also, if you do not have **do_verify()** overridden, and you do not have the correct sha-256 sum, then the expected value will be shown in the build output of your package.

```bash
pkg_shasum=36658cb768a54c1d4dec43c3116c27ed893e88b02ecfcb44f2166f9c0b7f2a0d
```

**pkg_deps**
: _Optional_. An array of package dependencies needed at runtime. You can refer to packages at three levels of specificity: origin/package, origin/package/version, or origin/package/version/release.

```bash
pkg_deps=(core/glibc core/pcre core/openssl core/zlib)
```

**pkg\_build\_deps**
: _Optional_. An array of the package dependencies needed only at build time.

```bash
pkg_build_deps=(core/gcc core/linux-headers)
```

**pkg\_lib\_dirs**
: _Optional_. An array of paths, relative to the final install of the software, where libraries can be found. Used to populate `LD_FLAGS` and `LD_RUN_PATH` for software that depends on your package.

```bash
pkg_lib_dirs=(lib)
```

**pkg\_include\_dirs**
: _Optional_. An array of paths, relative to the final install of the software, where headers can be found. Used to populate `CFLAGS` for software that depends on your package.

```bash
pkg_include_dirs=(include)
```

**pkg\_bin\_dirs**
: _Optional_. An array of paths, relative to the final install of the software, where binaries can be found. Used to populate `PATH` for software that depends on your package.

```bash
pkg_bin_dirs=(bin)
```

**pkg\_pconfig\_dirs**
: _Optional_. An array of paths, relative to the final install of the software, where pkg-config metadata (.pc files) can be found. Used to populate `PKG_CONFIG_PATH` for software that depends on your package.

```bash
pkg_pconfig_dirs=(lib/pkgconfig)
```

**pkg\_svc\_run**
: _Optional_. The command for the Supervisor to execute when starting a service. You can omit this setting if your package is not intended to be run directly by a Supervisor. If you set this you will need to set `pkg_bin_dirs` so that the binaries in your package will be in the
path

```bash
pkg_svc_run="haproxy -f $pkg_svc_config_path/haproxy.conf"
```

> **Note**: You should use a [run hook](#hooks) instead if you have complex start up behavior.

**pkg_exports**
: _Optional_. An [associative array](http://www.linuxjournal.com/content/bash-associative-arrays) (or `hashtable` in Powershell) representing configuration data which should be gossiped to peers. The keys in this array are used with `pkg_exposes` and for any consuming services that set `pkg_binds` or `pkg_binds_optional`. The values represent the TOML path to a value.

```bash
pkg_exports=(
  [port]=server.port
  [host]=server.host
  [ssl-port]=ssl.port
)
```

In this example, the corresponding default.toml file would have the following key/value pairs defined:

```toml default.toml
    [server]
    port = 80
    host = "www.example.com"

    [ssl]
    port = 443
```

**pkg_exposes**
: _Optional_. An array of `pkg_exports` keys containing default values for the ports that this package exposes. These values are used as sensible defaults for other tools, such as when exporting a package to a container format.

```bash
pkg_exposes=(port ssl-port)
```

  > **Note**: In addition to specifying the keys you defined in `pkg_exports`, you **must** have a default.toml file indicating the port values to expose.

**pkg_binds**
: _Optional_. An associative array (or `hashtable` in Powershell) representing services which you depend on and the configuration keys that you expect the service to export (by their `pkg_exports`). These binds *must* be set for the Supervisor to load the service. The loaded service will wait to run until its bind becomes available. If the bind does not contain the expected keys, the service will not start successfully.

```bash
pkg_binds=(
  [database]="port host"
)
```

**pkg\_binds\_optional**
: _Optional_. Same as `pkg_binds` but these represent optional services to connect to.

```bash
pkg_binds_optional=(
  [storage]="port host"
)
```

**pkg_interpreters**
: _Optional_. An array of interpreters used in [shebang](https://en.wikipedia.org/wiki/Shebang_(Unix)) lines for scripts. Specify the subdirectory where the binary is relative to the package, for example, `bin/bash` or `libexec/neverland`, since binaries can be located in directories besides `bin`. This list of interpreters will be written to the metadata INTERPRETERS file, located inside a package, with their fully-qualified path. Then these can be used with the fix_interpreter function. For more information on declaring shebangs in Chef Habitat, see [Plan hooks](#hooks), and for more information on the fix_interpreter function, see [Plan utility functions](#plan-helpers).

```bash
pkg_interpreters=(bin/bash)
```

**pkg\_svc\_user**
: _Optional_. The user to run the service as. The default is `hab`. On Windows, if the `hab` user does not exist then the service will run under the same account as the Supervisor.

```bash
pkg_svc_user=hab
```

**pkg\_svc\_group**
: _Optional_. The group to run the service as. The default is `hab`.

```bash
pkg_svc_group=$pkg_svc_user
```

> **Note**: `pkg_svc_group` is not used in a `plan.ps1`.

**pkg_shutdown_signal**
: _Optional_. The signal to send the service to shutdown. The default is `TERM`.

```bash
pkg_shutdown_signal=HUP
```

> **Note**: `pkg_shutdown_signal` is not used in a `plan.ps1`.

**pkg_shutdown_timeout_sec**
: _Optional_. The number of seconds to wait for a service to shutdown. After this interval the service will forcibly be killed. The default is `8`.

```bash
pkg_shutdown_timeout_sec=$pkg_shutdown_timeout_sec
```

**pkg_description**
: _Required_ for [core](https://github.com/habitat-sh/core-plans) plans, optional otherwise. A short description of the package. It can be a simple string, or you can create a multi-line description using markdown to provide a rich description of your package. **This description will be displayed on the Web app when users search for or browse to your package.**

```bash
pkg_description=$(cat << EOF
  # My package description
  This is the package for the foo library. It's pretty awesome.
  EOF
  )
```

> **Note**: Any special characters other than `#` will have to be escaped; otherwise, they could be interpreted by the hab-plan-build script when the package is built.

**pkg\_upstream\_url**
: _Optional_. An upstream project homepage or website URL.

```bash
pkg_upstream_url=https://github.com/myrepo
```
