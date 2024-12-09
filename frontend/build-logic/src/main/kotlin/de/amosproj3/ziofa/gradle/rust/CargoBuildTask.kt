// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.gradle.rust

import org.gradle.api.DefaultTask
import java.io.File
import javax.inject.Inject
import org.gradle.api.GradleException
import org.gradle.api.file.FileCollection
import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.Property
import org.gradle.api.provider.Provider
import org.gradle.api.tasks.CacheableTask
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFile
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.Optional
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import org.gradle.process.ExecOperations

@CacheableTask
abstract class CargoBuildTask @Inject constructor(private val execOperations: ExecOperations) :
    DefaultTask() {

  @get:Input abstract val cmdlineArgs: ListProperty<String>

  @get:Input abstract val workspaceDirectory: Property<File>

  @get:OutputFile abstract val outputFile: Property<File>

  @get:InputFile
  @get:PathSensitive(PathSensitivity.ABSOLUTE)
  internal val cargo: Provider<File> = project.provider { findCargo() }

  @get:InputFiles
  @get:Optional
  @get:PathSensitive(PathSensitivity.RELATIVE)
  internal val sources: Provider<FileCollection> by lazy {
    workspaceDirectory.map { project.fileTree(it) { exclude("target") } }
  }

  @TaskAction
  open fun run() {
    val workspace = workspaceDirectory.get()
    val cargo = cargo.get()
    val args = cmdlineArgs.get()

    execOperations.exec {
      workingDir = workspace

      commandLine(cargo, *args.toTypedArray())
    }
  }
}

private fun findCargo(): File {
  val paths = System.getenv("PATH")?.split(File.pathSeparator) ?: emptyList()
  return paths.asSequence().map { File(it, "cargo") }.firstOrNull { it.exists() && it.canExecute() }
      ?: throw GradleException("cargo is not installed")
}
