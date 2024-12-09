// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.gradle.rust

import org.gradle.api.DefaultTask
import java.io.File
import javax.inject.Inject
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.file.FileCollection
import org.gradle.api.provider.Property
import org.gradle.api.provider.Provider
import org.gradle.api.tasks.CacheableTask
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFile
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.Optional
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import org.gradle.process.ExecOperations

@CacheableTask
abstract class UniffiBindgenTask @Inject constructor(private var execOperations: ExecOperations) :
    DefaultTask() {

  @get:InputFile @get:PathSensitive(PathSensitivity.NONE) abstract val bindgenCli: Property<File>

  @get:InputFile @get:PathSensitive(PathSensitivity.NONE) abstract val library: Property<File>

  @get:OutputDirectory abstract val outDir: DirectoryProperty

  @get:Input abstract val workspace: Property<File>

  @get:InputFiles
  @get:Optional
  @get:PathSensitive(PathSensitivity.RELATIVE)
  internal val sources: Provider<FileCollection> by lazy {
    workspace.map { project.fileTree(it) { exclude("target") } }
  }

  @TaskAction
  fun run() {
    val library = library.get()
    val bindgen = bindgenCli.get()
    val outDir = outDir.get()
    val workspace = workspace.get()

    execOperations.exec {
      workingDir = workspace

      commandLine(
          bindgen, "generate", "--language=kotlin", "--library", "--out-dir=$outDir", library)
    }
  }
}
