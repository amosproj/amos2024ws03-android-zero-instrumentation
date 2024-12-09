// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.gradle.rust

import org.gradle.api.DefaultTask
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.file.FileSystemOperations
import org.gradle.api.tasks.InputDirectory
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.TaskAction
import javax.inject.Inject

abstract class BridgeTask @Inject constructor(
  private var fileSystemOperations: FileSystemOperations
) : DefaultTask() {

  @get:InputDirectory abstract val inputDirectory: DirectoryProperty

  @get:OutputDirectory abstract val outputDirectory: DirectoryProperty

  @TaskAction
  fun run() {
    fileSystemOperations.copy {
      from(inputDirectory.get())
      into(outputDirectory.get())
    }
  }
}
