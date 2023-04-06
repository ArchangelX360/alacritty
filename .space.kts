import circlet.pipelines.script.KotlinScriptSimpleFailureException
import circlet.pipelines.script.ScriptApi
import kotlin.io.path.Path
import kotlin.io.path.readText
import kotlin.io.path.writeText

private val alacrittyTerminalCargoTomlRelativePathString = "alacritty_terminal/Cargo.toml"

/**
 * The path to the Cargo.toml of the alacritty_terminal package.
 */
private val alacrittyTerminalCargoToml = Path(alacrittyTerminalCargoTomlRelativePathString)

job("Publish on Fleet tag") {
    startOn {
        gitPush {
            anyTagMatching {
                +"v*-fleet*"
            }
        }
    }

    host(displayName = "Publish `alacritty_terminal`") {
        kotlinScript(displayName = "Bump version in $alacrittyTerminalCargoTomlRelativePathString") { api ->
            val newVersion = api.versionFromFleetTag()
            val oldContent = alacrittyTerminalCargoToml.readText()
            val newContent = oldContent.replacePackageVersion("alacritty_terminal", newVersion)
            if (oldContent == newContent) {
                throw KotlinScriptSimpleFailureException("failed to replace version, has the format of Cargo.toml changed?")
            }
            alacrittyTerminalCargoToml.writeText(newContent)
            println("Will publish package using the following $alacrittyTerminalCargoToml:\n${alacrittyTerminalCargoToml.readText()}")
        }

        shellScript(displayName = "Locally commit version bump") {
            content = """
                set -xe
                git add $alacrittyTerminalCargoTomlRelativePathString
                git commit -m 'this will not be commited but prevent usage of --allow-dirty in cargo publish command'
            """
        }

        shellScript(displayName = "Install Rust tooling") {
            content = """
                set -xe
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
            """
        }

        env["CARGO_PUBLISH_TOKEN"] = "{{ project:fleet_alacritty_fork_publisher_fleet-crates_write_token }}"
        shellScript("Publish `alacritty_terminal` crate to Space registry") {
            content = """
                set -xe
                export HOME=/root
                . ${'$'}HOME/.cargo/env
                cargo login --registry=space-fleet-crates ${'$'}CARGO_PUBLISH_TOKEN
                cargo publish --registry=space-fleet-crates --package alacritty_terminal
            """
        }
    }
}

fun ScriptApi.versionFromFleetTag(): String {
    val versionRegex = Regex("refs/tags/v(?<version>.*-fleet.*)")
    return versionRegex.matchEntire(gitBranch())?.groups?.get(1)?.value
        ?: throw KotlinScriptSimpleFailureException("triggered on ref ${gitBranch()} which does not match $versionRegex, you may have made a mistake in your tag format, otherwise if you are trying to manually run this job, don't :).")
}

fun String.replacePackageVersion(packageName: String, newVersion: String): String {
    val filePrefixUntilVersionRegex = Regex(
        """
        \[package\]
        name = "$packageName"
        version = "(?<version>[^"]*)"
        """.trimIndent()
    )
    val filePrefixWithNewVersion = """
                [package]
                name = "$packageName"
                version = "$newVersion"
            """.trimIndent()
    return replace(filePrefixUntilVersionRegex, filePrefixWithNewVersion)
}
